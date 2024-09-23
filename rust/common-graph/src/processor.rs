use crate::{
    storage::{GraphData, GraphStorage},
    CommonGraphError, Graph, Node, PortType, Result,
};
use std::{collections::VecDeque, fmt::Debug};

/// The item yielded by [`GraphProcessor`]'s iteration.
///
/// It's lifetime is valid only until the next
/// [`GraphProcessor::try_next`] call.
pub struct GraphProcessorItem<'a, T, V>
where
    T: Debug + 'a,
    V: Clone + 'a,
{
    node: &'a Node<T>,
    inputs: &'a Vec<(&'a str, Option<V>)>,
    outputs: Vec<(&'a str, &'a mut Option<V>)>,
}

impl<'a, T, V> GraphProcessorItem<'a, T, V>
where
    T: Debug + 'a,
    V: Clone + 'a,
{
    fn new(
        node: &'a Node<T>,
        inputs: &'a Vec<(&'a str, Option<V>)>,
        outputs: Vec<(&'a str, &'a mut Option<V>)>,
    ) -> Self {
        GraphProcessorItem {
            node,
            inputs,
            outputs,
        }
    }

    /// Returns the [`Node`] currently being processed.
    pub fn node(&self) -> &Node<T> {
        self.node
    }

    /// Returns the map of port name and value, representing
    /// incoming data.
    pub fn inputs(&self) -> &[(&'a str, Option<V>)] {
        &self.inputs[..]
    }

    /// Returns the map of port name and value, representing
    /// outgoing data. The data is mutable and able to be written
    /// to.
    pub fn outputs_mut(&'a mut self) -> &'a mut [(&'a str, &'a mut Option<V>)] {
        &mut self.outputs[..]
    }
}

/// An iterating processor for [`Graph`].
///
/// A [`GraphProcessor`] takes inputs, and feeds them into an immutable [`Graph`]
/// network. Using [`GraphProcessor::try_next`], once a node has all of its
/// inputs fulfilled, the processor yields a node and its I/O data,
/// where its output can be written. The node's output data propagates
/// through the network, and the next node with complete input yields,
/// and so forth, until all nodes have been processed.
///
/// A [`GraphProcessor`] is similar to an [`Iterator`],
/// using the [lending iterator] pattern instead to "lend"
/// mutable references with a lifetime valid until the
/// subsequent [`GraphProcessor::try_next`] call.
///
/// [lending iterator]: <https://docs.rs/lending-iterator>
pub struct GraphProcessor<'g, T, V>
where
    T: Debug + 'g,
    V: Clone + 'g,
    Self: 'g,
{
    graph: &'g Graph<T>,
    current_index: Option<usize>,
    storage: GraphStorage<'g, V>,
    queue: VecDeque<usize>,
}

impl<'g, T, V> GraphProcessor<'g, T, V>
where
    T: Debug + 'g,
    V: Clone + 'g,
    Self: 'g,
{
    /// Creates a new [`GraphProcessor`].
    pub(crate) fn new<'ext, I>(graph: &'g Graph<T>, input: I) -> Result<Self>
    where
        I: IntoIterator<Item = (&'ext str, V)>,
    {
        let mut queue = VecDeque::default();

        // Seed the storage with port keys and queue up any
        // input-less nodes first after the root node.
        let mut storage = GraphStorage::from_iter(graph.nodes().iter().map(|node| {
            if !node.is_root() && node.inputs().is_empty() {
                queue.push_back(node.index())
            }
            return (
                node.inputs().iter().map(|(key, _)| key.as_str()),
                node.outputs().iter().map(|(key, _)| key.as_str()),
            );
        }));

        // Seed input data on root node outputs
        for (port, value) in input {
            storage.set(0, port, value, PortType::Output)?;
        }

        Ok(Self {
            graph,
            storage,
            current_index: Some(0),
            queue,
        })
    }
}

/// The lending iterator type trait implemented by [`GraphProcessor`].
pub trait Processor {
    /// Type of item to yield on [`Processor::try_next`]
    type Item<'a>
    where
        Self: 'a;

    /// Type of finalized output.
    type Output;

    /// Yield the next node to process. If `None`,
    /// there are no more items to process.
    fn try_next(&mut self) -> Result<Option<Self::Item<'_>>>;

    /// Consumes the [`Processor`], returning the output of the
    /// process if it exists. Will be `None` unless
    /// [`Processor::try_next`] has been iteratively exhausted.
    fn output(self) -> Option<Self::Output>;
}

impl<'g, T, V> Processor for GraphProcessor<'g, T, V>
where
    T: Debug,
    V: Clone,
{
    type Item<'next>
        = GraphProcessorItem<'next, T, V>
    where
        Self: 'next;

    type Output = GraphData<'g, V>;

    fn try_next(&'_ mut self) -> Result<Option<GraphProcessorItem<'_, T, V>>> {
        // First we take the previously processed node
        // that had an opportunity to write outputs and propagate
        // values to outgoing connections.
        let Some(previous_index) = self.current_index else {
            // We are done iterating.
            return Ok(None);
        };
        let previous_node = self.graph.get_node(previous_index)?;

        // Pump outgoing data to inputs
        for (port_name, outgoing) in previous_node.outputs().iter() {
            let Some(outgoing) = outgoing else {
                continue;
            };

            let value = match self
                .storage
                .get(previous_index, port_name, PortType::Output)
            {
                Ok(Some(value)) => Ok(value.to_owned()),
                Ok(None) => Err(CommonGraphError::Unexpected(
                    "Expected output port to have value.".into(),
                )),
                Err(e) => Err(e),
            }?;

            for (outgoing_index, outgoing_port) in outgoing {
                let outgoing_index = outgoing_index.to_owned();
                self.storage.set(
                    outgoing_index,
                    outgoing_port,
                    value.to_owned(),
                    PortType::Input,
                )?;

                // If this node's output provides the final input
                // needed to the outgoing node, queue it up.
                // Do not queue up the root again once its inputs
                // are full -- the graph has been fully processed.
                if outgoing_index != 0 && self.storage.is_full(outgoing_index, PortType::Input) {
                    self.queue.push_back(outgoing_index);
                }
            }
        }

        // Queue up the next node to process.
        let Some(next_index) = self.queue.pop_front() else {
            // No more nodes to queue -- collect output.
            self.current_index = None;
            return Ok(None);
        };

        self.current_index = Some(next_index);
        let next_node = self.graph.get_node(next_index)?;

        if next_node.is_root() {
            return Err(CommonGraphError::Unexpected(
                "Root node erroneously queued.".into(),
            ));
        }

        if !self.storage.is_full(next_index, PortType::Input) {
            return Err(CommonGraphError::Unexpected(
                "Node queued without full inputs.".into(),
            ));
        }

        let (inputs, outputs) = self.storage.get_io_mut(next_index)?;

        Ok(Some(GraphProcessorItem::new(next_node, inputs, outputs)))
    }

    fn output(self) -> Option<Self::Output> {
        if self.current_index.is_none() {
            Some(self.storage.into())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        helpers::{Op, OpNum},
        GraphBuilder,
    };
    use common_tracing::common_tracing;

    #[test]
    #[common_tracing]
    fn it_processes_graph_linear_equation() -> Result<()> {
        // ax + b
        let graph = GraphBuilder::default()
            .set_graph_input(["a", "b"])
            .set_graph_output(["out"])
            .node(
                "identity",
                Op::Identity(-1.0),
                Vec::<String>::new(),
                ["out"],
            )
            .node("flip", Op::Multiply, ["x", "y"], ["out"])
            .node("divide", Op::Divide, ["x", "y"], ["out"])
            .connect_input("a", ("divide", "y"))?
            .connect_input("b", ("flip", "x"))?
            .connect(("identity", "out"), ("flip", "y"))?
            .connect(("flip", "out"), ("divide", "x"))?
            .connect_output(("divide", "out"), "out")?
            .build()?;

        // 10x - 5
        let output = graph.process([("a", 10.0), ("b", -5.0)], Op::run)??;
        assert_eq!(output.inner()[0].0, vec![("out", Some(0.5))]);
        // -2x + 9
        let output = graph.process([("a", -2.0), ("b", 9.0)], Op::run)??;
        assert_eq!(output.inner()[0].0, vec![("out", Some(4.5))]);
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_processes_graph_rgb_to_hsv() -> Result<()> {
        // Modeled after python's `colorsys`:
        // https://github.com/python/cpython/blob/d5abd02f36bbee5720944b8906a118a8fb66d75b/Lib/colorsys.py#L75
        let graph = GraphBuilder::default()
            .set_graph_input(["r", "g", "b"])
            .set_graph_output(["h", "s", "v"])
            .node("maxc", Op::Max, ["r", "g", "b"], ["out"])
            .node("minc", Op::Min, ["r", "g", "b"], ["out"])
            .node("rangec", Op::Subtract, ["maxc", "minc"], ["out"])
            .node("maxc_r", Op::Subtract, ["maxc", "r"], ["out"])
            .node("maxc_g", Op::Subtract, ["maxc", "g"], ["out"])
            .node("maxc_b", Op::Subtract, ["maxc", "b"], ["out"])
            .node("saturation", Op::Divide, ["rangec", "maxc"], ["out"])
            .node("rc", Op::Divide, ["maxc_r", "rangec"], ["out"])
            .node("gc", Op::Divide, ["maxc_g", "rangec"], ["out"])
            .node("bc", Op::Divide, ["maxc_b", "rangec"], ["out"])
            .node("r_eq_maxc", Op::Eq, ["r", "maxc"], ["out"])
            .node("g_eq_maxc", Op::Eq, ["g", "maxc"], ["out"])
            .node("one", Op::Identity(1.0), Vec::<String>::new(), ["out"])
            .node("two", Op::Identity(2.0), Vec::<String>::new(), ["out"])
            .node("four", Op::Identity(4.0), Vec::<String>::new(), ["out"])
            .node("six", Op::Identity(6.0), Vec::<String>::new(), ["out"])
            .node("h_branch1", Op::Subtract, ["bc", "gc"], ["out"])
            .node("h_branch2_tmp", Op::Subtract, ["rc", "bc"], ["out"])
            .node("h_branch2", Op::Add, ["two", "h_branch2_tmp"], ["out"])
            .node("h_branch3_tmp", Op::Subtract, ["gc", "rc"], ["out"])
            .node("h_branch3", Op::Add, ["four", "h_branch3_tmp"], ["out"])
            .node("if_g_maxc", Op::IfThenElse, ["if", "then", "else"], ["out"])
            .node("if_r_maxc", Op::IfThenElse, ["if", "then", "else"], ["out"])
            .node("h_divide", Op::Divide, ["h", "six"], ["out"])
            .node("h_mod", Op::Modulo, ["h", "one"], ["out"])
            .connect_input("r", ("maxc", "r"))?
            .connect_input("g", ("maxc", "g"))?
            .connect_input("b", ("maxc", "b"))?
            .connect_input("r", ("minc", "r"))?
            .connect_input("g", ("minc", "g"))?
            .connect_input("b", ("minc", "b"))?
            .connect_input("r", ("maxc_r", "r"))?
            .connect_input("g", ("maxc_g", "g"))?
            .connect_input("b", ("maxc_b", "b"))?
            .connect_input("r", ("r_eq_maxc", "r"))?
            .connect_input("g", ("g_eq_maxc", "g"))?
            .connect(("maxc", "out"), ("rangec", "maxc"))?
            .connect(("minc", "out"), ("rangec", "minc"))?
            .connect(("maxc", "out"), ("maxc_r", "maxc"))?
            .connect(("maxc", "out"), ("maxc_g", "maxc"))?
            .connect(("maxc", "out"), ("maxc_b", "maxc"))?
            .connect(("maxc_r", "out"), ("rc", "maxc_r"))?
            .connect(("maxc_g", "out"), ("gc", "maxc_g"))?
            .connect(("maxc_b", "out"), ("bc", "maxc_b"))?
            .connect(("rangec", "out"), ("rc", "rangec"))?
            .connect(("rangec", "out"), ("gc", "rangec"))?
            .connect(("rangec", "out"), ("bc", "rangec"))?
            .connect(("rangec", "out"), ("saturation", "rangec"))?
            .connect(("maxc", "out"), ("saturation", "maxc"))?
            .connect(("maxc", "out"), ("r_eq_maxc", "maxc"))?
            .connect(("maxc", "out"), ("g_eq_maxc", "maxc"))?
            .connect(("bc", "out"), ("h_branch1", "bc"))?
            .connect(("gc", "out"), ("h_branch1", "gc"))?
            .connect(("rc", "out"), ("h_branch2_tmp", "rc"))?
            .connect(("bc", "out"), ("h_branch2_tmp", "bc"))?
            .connect(("two", "out"), ("h_branch2", "two"))?
            .connect(("h_branch2_tmp", "out"), ("h_branch2", "h_branch2_tmp"))?
            .connect(("gc", "out"), ("h_branch3_tmp", "gc"))?
            .connect(("rc", "out"), ("h_branch3_tmp", "rc"))?
            .connect(("four", "out"), ("h_branch3", "four"))?
            .connect(("h_branch3_tmp", "out"), ("h_branch3", "h_branch3_tmp"))?
            .connect(("g_eq_maxc", "out"), ("if_g_maxc", "if"))?
            .connect(("h_branch2", "out"), ("if_g_maxc", "then"))?
            .connect(("h_branch3", "out"), ("if_g_maxc", "else"))?
            .connect(("r_eq_maxc", "out"), ("if_r_maxc", "if"))?
            .connect(("h_branch1", "out"), ("if_r_maxc", "then"))?
            .connect(("if_g_maxc", "out"), ("if_r_maxc", "else"))?
            .connect(("if_r_maxc", "out"), ("h_divide", "h"))?
            .connect(("six", "out"), ("h_divide", "six"))?
            .connect(("h_divide", "out"), ("h_mod", "h"))?
            .connect(("one", "out"), ("h_mod", "one"))?
            .connect_output(("h_mod", "out"), "h")?
            .connect_output(("maxc", "out"), "v")?
            .connect_output(("saturation", "out"), "s")?
            .build()?;

        // Expected values come from python implementation
        for (r, g, b, expected) in [
            (1.0, 0.5, 0.1, (0.07407407, 0.9, 1.0)),
            (0.5, 1.0, 0.1, (0.25925925, 0.9, 1.0)),
            (0.5, 0.9, 1.0, (0.53333336, 0.5, 1.0)),
        ] {
            let output = graph.process([("r", r), ("g", g), ("b", b)], Op::run)??;
            let result: Vec<OpNum> = output.into_inner()[0]
                .0
                .iter()
                .map(|(_, value)| value.unwrap())
                .collect();
            assert_eq!((result[0], result[1], result[2]), expected);
        }

        Ok(())
    }

    #[common_tracing]
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn it_processes_graph_async_fibonacci() -> Result<()> {
        #[derive(Clone, PartialEq, Debug)]
        struct Number(u32);

        /// Calculates the nth fibonacci number,
        /// where f(0) = 0, f(1) = 1, f(2) = 2
        async fn fibonacci(n: &Number) -> Result<Number> {
            let inner = n.0;
            Ok(tokio::task::spawn_blocking(move || {
                fn fibonacci(n: u32) -> u32 {
                    match n {
                        0 => 0,
                        1 | 2 => 1,
                        _ => fibonacci(n - 1) + fibonacci(n - 2),
                    }
                }
                fibonacci(inner)
            })
            .await
            .map(|n| Number(n))
            .map_err(|_| CommonGraphError::from("Error on blocking thread"))?)
        }

        let nodes = [
            (None, "Root", (vec!["in".into()], vec!["out".into()])),
            (
                Some(()),
                "Fibonacci",
                (vec!["in".into()], vec!["out".into()]),
            ),
        ];
        let edges = [
            ((0, "out".into()), (1, "in".into())),
            ((1, "out".into()), (0, "in".into())),
        ];

        async fn callback<'a, 'b>(
            item: &'a mut GraphProcessorItem<'b, (), Number>,
        ) -> ::std::result::Result<(), CommonGraphError>
        where
            'a: 'b,
        {
            let input = item.inputs()[0].1.clone().unwrap();
            let result = fibonacci(&input).await?;
            let out_port: &mut (&str, &mut Option<Number>) = item
                .outputs_mut()
                .iter_mut()
                .find(|(key, _)| *key == "out")
                .unwrap();
            *out_port.1 = Some(result);
            Ok(())
        }

        let input = Number(13);
        let expected = Number(233);
        let graph = Graph::new(nodes, edges)?;

        let mut processor = graph.process_iter([("out".into(), input)])?;
        while let Some(mut item) = processor.try_next()? {
            callback(&mut item).await?;
        }
        let output = processor
            .output()
            .ok_or_else(|| CommonGraphError::Unexpected("Graph does not have output.".into()))?;
        // let output = graph.process_async([("out".into(), input)], callback).await??;
        assert_eq!(output.into_inner()[0].0[0].1, Some(expected));
        Ok(())
    }
}
