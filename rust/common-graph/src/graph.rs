use super::PortDetails;
use super::{CommonGraphError, ConnectionDetails, Result};
use crate::storage::GraphData;
use crate::utils::{is_empty, non_unique_entries};
use crate::{GraphProcessor, GraphProcessorItem, Processor};
use std::collections::HashSet;
use std::fmt::Debug;

/// A reference to a port via the node index and port name.
pub type PortRef = (usize, String);
/// An edge in a graph containing the source `0` and target `1` ports.
pub type Edge = (PortRef, PortRef);

/// Representing a distinction between input and output ports.
#[derive(strum::Display, Debug, PartialEq)]
pub enum PortType {
    /// An input port.
    #[strum(to_string = "input")]
    Input,
    /// An output port.
    #[strum(to_string = "output")]
    Output,
}

/// The internal node of a [`Graph`].
#[derive(Debug)]
pub struct Node<T> {
    index: usize,
    inner: Option<T>,
    label: String,
    inputs: Vec<(String, Option<PortRef>)>,
    outputs: Vec<(String, Option<Vec<PortRef>>)>,
}

impl<T> Node<T> {
    pub(crate) fn new(
        index: usize,
        inner: Option<T>,
        label: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) -> Result<Self> {
        for (port_type, ports) in [(PortType::Input, &inputs), (PortType::Output, &outputs)] {
            if let Some(dupe) = non_unique_entries(ports) {
                return Err(CommonGraphError::DuplicatePorts(
                    PortDetails {
                        node: label,
                        port_type,
                        port: dupe.to_owned(),
                    }
                    .into(),
                ));
            }
        }

        let inputs = inputs.into_iter().map(|s| (s, None)).collect();
        let outputs = outputs.into_iter().map(|s| (s, None)).collect();
        Ok(Self {
            index,
            inner,
            label,
            inputs,
            outputs,
        })
    }

    /// The index of this node in the graph.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Whether this is the root node, representing graph input/output.
    pub fn is_root(&self) -> bool {
        self.inner.is_none()
    }

    /// Returns the inner `T` for this node.
    pub fn inner(&self) -> Option<&T> {
        self.inner.as_ref()
    }

    /// Returns the string label for this node.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get incoming ports with their source [`PortRef`].
    pub fn inputs(&self) -> &[(String, Option<PortRef>)] {
        &self.inputs[..]
    }

    /// Get outgoing ports with their target [`PortRef`].
    pub fn outputs(&self) -> &[(String, Option<Vec<PortRef>>)] {
        &self.outputs[..]
    }

    fn set_outgoing(&mut self, port_name: &str, target_ref: PortRef) -> Result<()> {
        let Some(port) = self.outputs.iter_mut().find(|(s, _)| s == port_name) else {
            return Err(CommonGraphError::MissingPort(
                PortDetails {
                    node: self.label().into(),
                    port_type: PortType::Output,
                    port: port_name.into(),
                }
                .into(),
            ))?;
        };

        match &mut port.1 {
            Some(ref mut vec) => {
                vec.push(target_ref);
            }
            None => {
                port.1 = Some(vec![target_ref]);
            }
        };
        Ok(())
    }

    fn set_incoming(&mut self, port_name: &str, port_ref: PortRef) -> Result<()> {
        let Some(port) = self.inputs.iter_mut().find(|(s, _)| s == port_name) else {
            return Err(CommonGraphError::MissingPort(
                PortDetails {
                    node: self.label().into(),
                    port_type: PortType::Input,
                    port: port_name.into(),
                }
                .into(),
            ))?;
        };

        match &mut port.1 {
            // Port has an existing incoming connection
            Some(_) => Err(CommonGraphError::MultipleInputs(
                PortDetails {
                    node: self.label().into(),
                    port: port_name.into(),
                    port_type: PortType::Input,
                }
                .into(),
            )),
            // Port exists and has no outgoing connection
            None => {
                port.1 = Some(port_ref);
                Ok(())
            }
        }
    }
}

/// A [directed acyclic graph] of nodes with ports.
///
/// The [`Graph`] contains vertices that have named
/// input and output ports with edges represented by a directed
/// connection from an output port to an input port. The graph itself
/// has inputs and outputs, represented by a root node's outputs and
/// inputs respectively, that form the basis of feeding inputs
/// through the graph via [`Graph::process_iter`], generating output.
///
/// Constructing a new graph from [`Graph::new`] takes iterators
/// for nodes and edges.
/// When constructing the graph from its inputs, the following
/// properties must hold true:
///
/// * Node labels are unique across all nodes
/// * Port names are unique across ports of the
///   same type in a node (two output ports can't have the same name,
///   but an input and output port can).
/// * No cycles other than the implicit cycle created by graph
///   IO and the root node. Nodes can only connect to nodes of
///   a higher index.
/// * Nodes must have at least one incoming or outgoing connection.
/// * All connections described in the input must be valid.
/// * Input ports may only have one incoming connection (no fan-in).
/// * Edges must be unique across the graph. Implicitly
///   enforced by not allowing fan-in.
/// * There must be a single root node, and it must be located at index `0`.
/// * Graph must contain at least one root node and one other connected node.
///
/// # Examples
///
/// ```
/// use common_graph::{Graph, Processor, Result};
///
/// # fn main() -> Result<()> {
/// let nodes = [
///     (None, "Root", (vec!["in".into()], vec!["out".into()])),
///     (Some(String::from("sq")), "Square", (vec!["in".into()], vec!["out".into()])),
/// ];
/// let edges = [
///     ((0, "out".into()), (1, "in".into())),
///     ((1, "out".into()), (0, "in".into())),
/// ];
///
/// let graph = Graph::new(nodes, edges)?;
/// let mut processor = graph.process_iter([("out".into(), 5u8)])?;
/// while let Some(mut item) = processor.try_next()? {
///     let inner = item.node().inner().unwrap();
///     let input = item.inputs()[0].1.unwrap();
///     let out = match inner.as_str() {
///       "sq" => input * input,
///       _ => 0,
///     };
///     let out_port: &mut (&str, &mut Option<u8>) = item
///         .outputs_mut()
///         .iter_mut()
///         .find(|(key, _)| *key == "out")
///         .unwrap();
///    *out_port.1 = Some(out);
/// }
/// let root_out = processor.output().unwrap()
///     .into_inner()
///     [0].0.clone();
/// assert_eq!(root_out, vec![("in", Some(25))]);
/// # Ok(())
/// # }
/// ```
/// [directed acyclic graph]: https://en.wikipedia.org/wiki/Directed_acyclic_graph
#[derive(Debug, Default)]
pub struct Graph<T: Debug> {
    nodes: Vec<Node<T>>,
}

impl<T> Graph<T>
where
    T: Debug,
{
    /// Takes an iterator of tuples of nodes containing an
    /// inner `T`, a label used in debugging, and its input
    /// and output port names. The iterator of edges is used to
    /// populate all connections. Fails if graph is invalid.
    ///
    /// See [`Graph`] for all constraints.
    pub fn new<NodeIter, EdgeIter, S>(nodes_iter: NodeIter, edges: EdgeIter) -> Result<Self>
    where
        NodeIter: IntoIterator<Item = (Option<T>, S, (Vec<String>, Vec<String>))>,
        EdgeIter: IntoIterator<Item = Edge>,
        S: Into<String>,
    {
        let mut nodes = vec![];
        let mut node_set: HashSet<String> = HashSet::default();
        for (index, (inner, label, io)) in nodes_iter.into_iter().enumerate() {
            let label = label.into();
            if !node_set.insert(label.clone()) {
                return Err(CommonGraphError::DuplicateLabels(label.clone()));
            }
            let node = Node::new(index, inner, label, io.0, io.1)?;
            if index == 0 && !node.is_root() {
                return Err(CommonGraphError::InvalidRoot);
            }
            if index != 0 && node.is_root() {
                return Err(CommonGraphError::InvalidRoot);
            }
            nodes.push(node);
        }

        // A graph that doesn't have both a root node and one other node
        // is considered empty.
        if nodes.len() < 2 {
            return Err(CommonGraphError::EmptyGraph);
        }

        populate_connections(&mut nodes, edges)?;

        for node in nodes.iter() {
            if is_empty(node.inputs()) && is_empty(node.outputs()) {
                return Err(CommonGraphError::OrphanedNode(node.label().into()));
            }
        }

        Ok(Graph { nodes })
    }

    /// Returns a reference to nodes in this graph.
    pub fn nodes(&self) -> &Vec<Node<T>> {
        &self.nodes
    }

    /// Returns a reference to an internal [`Node`].
    pub fn get_node(&self, index: usize) -> Result<&Node<T>> {
        self.nodes
            .get(index)
            .ok_or_else(|| CommonGraphError::MissingNode(index))
    }

    /// Returns a [`GraphProcessor`] that can be iterated over
    /// to propagate `input` throughout the graph network.
    pub fn process_iter<'a, 'ext, V, I>(&'a self, input: I) -> Result<GraphProcessor<'a, T, V>>
    where
        I: IntoIterator<Item = (&'ext str, V)>,
        V: Clone,
        'ext: 'a,
    {
        GraphProcessor::new(self, input)
    }

    /// Iteratively process this graph.
    ///
    /// `func` is called for each node with its inputs
    /// in order to write its outputs.
    ///
    /// # Examples
    ///
    /// ```
    /// use common_graph::{GraphProcessorItem, Graph, Result};
    ///
    /// # fn main() -> Result<()> {
    /// #[derive(Debug)]
    /// enum Op {
    ///   Square,
    /// }
    ///
    /// let nodes = [
    ///     (None, "Root", (vec!["in".into()], vec!["out".into()])),
    ///     (Some(Op::Square), "Square", (vec!["in".into()], vec!["out".into()])),
    /// ];
    /// let edges = [
    ///     ((0, "out".into()), (1, "in".into())),
    ///     ((1, "out".into()), (0, "in".into())),
    /// ];
    ///
    /// fn callback<'a>(
    ///    item: &'a mut GraphProcessorItem<'a, Op, f32>,
    /// ) -> ::std::result::Result<(), String> {
    ///     let v: Vec<_> = item
    ///         .inputs()
    ///         .iter()
    ///         .filter_map(|(_, value)| value.as_ref().cloned())
    ///         .collect();
    ///     let inner = item.node().inner().ok_or(String::from("Missing node"))?;
    ///     let result = match inner {
    ///         Op::Square => v[0] * v[0],
    ///     };
    ///     let out_port: &mut (&str, &mut Option<f32>) = item
    ///         .outputs_mut()
    ///         .iter_mut()
    ///         .find(|(key, _)| *key == "out")
    ///         .unwrap();
    ///    *out_port.1 = Some(result);
    ///     Ok(())
    /// };
    ///
    /// let graph = Graph::new(nodes, edges)?;
    /// let result = graph.process([("out", 5f32)], callback)??
    ///     .into_inner();
    /// let root_out = result[0].0.clone();
    /// assert_eq!(root_out, vec![("in", Some(25f32))]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn process<'a, 'ext, I, V, F, E>(
        &'a self,
        inputs: I,
        mut func: F,
    ) -> Result<::std::result::Result<GraphData<'a, V>, E>>
    where
        F: for<'b> FnMut(&'b mut GraphProcessorItem<'b, T, V>) -> ::std::result::Result<(), E>,
        I: IntoIterator<Item = (&'ext str, V)>,
        V: Clone + 'a,
        T: Debug + 'a,
        'ext: 'a,
    {
        let mut processor = self.process_iter(inputs)?;
        while let Some(mut item) = processor.try_next()? {
            if let Err(e) = func(&mut item) {
                return Ok(Err(e));
            };
        }
        let Some(output) = processor.output() else {
            return Err(CommonGraphError::Unexpected(
                "Graph does not have output.".into(),
            ))?;
        };
        Ok(Ok(output))
    }

    /*
    /// Iteratively process this graph with async callback.
    ///
    /// `func` is called for each node with its inputs
    /// in order to write its outputs.
    pub async fn process_async<'a, 'ext, I, V, F, Fut, E>(
        &'a self,
        inputs: I,
        mut func: F,
    ) -> Result<::std::result::Result<Vec<PortStore<'a, V>>, E>>
    where
        Fut: ::std::future::Future<Output = ::std::result::Result<(), E>>,
        F: for<'b> FnMut(&'b mut GraphProcessorItem<'b, T, V>) -> Fut,
        I: IntoIterator<Item = (&'ext str, V)>,
        V: Clone + 'a,
        T: Debug + 'a,
        'ext: 'a,
    {
        // TODO
        let mut processor = self.process_iter(inputs)?;
        while let Some(mut item) = processor.try_next()? {
            if let Err(e) = func(&mut item).await {
                return Ok(Err(e));
            };
        }
        let Some(output) = processor.output() else {
            return Err(CommonGraphError::Unexpected(
                "Graph does not have output.".into(),
            ))?;
        };
        Ok(Ok(output))
    }
    */
}

/// Takes the definition of edges and populates the
/// nodes with the appropriate references. If a graph invalidation
/// error occurs, the mutations are not reverted and the graph
/// should be considered invalid.
/// Called during construction.
fn populate_connections<T, E>(nodes: &mut [Node<T>], edges: E) -> Result<()>
where
    E: IntoIterator<Item = Edge>,
{
    for (source, target) in edges {
        let (source_index, source_port) = source.clone();
        let (target_index, target_port) = target.clone();

        // Perform some up-front checking to see that the
        // nodes exist, and get their label for error reporting.
        let (source_label, target_label) = {
            let Some(source_node) = nodes.get(source_index) else {
                return Err(CommonGraphError::MissingNode(source_index));
            };
            let Some(target_node) = nodes.get(target_index) else {
                return Err(CommonGraphError::MissingNode(target_index));
            };
            (source_node.label(), target_node.label())
        };

        // Cycles may only occur when threading back through the
        // root node. A quick way to ensure no cycles is only allowing
        // nodes to connect to nodes of a higher index.
        // If a node is connecting to itself, or to a node of a lower index,
        // consider it an invalid cycle.
        if target_index != 0 && source_index >= target_index {
            return Err(CommonGraphError::InvalidCycleDetected(
                ConnectionDetails {
                    source_node: source_label.into(),
                    source_port,
                    target_node: target_label.into(),
                    target_port,
                }
                .into(),
            ));
        }

        // Add connections
        for (index, port_name, port_ref, port_type) in [
            (source_index, source_port, target, PortType::Output),
            (target_index, target_port, source, PortType::Input),
        ] {
            let Some(node) = nodes.get_mut(index) else {
                return Err(CommonGraphError::MissingNode(index));
            };
            match port_type {
                PortType::Output => node.set_outgoing(&port_name, port_ref)?,
                PortType::Input => node.set_incoming(&port_name, port_ref)?,
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphBuilder;
    use common_tracing::common_tracing;

    /// Generate a vec of nodes and edges as a minimal supported graph.
    fn gen_graph_components() -> (
        Vec<(Option<()>, String, (Vec<String>, Vec<String>))>,
        Vec<((usize, String), (usize, String))>,
    ) {
        let nodes = vec![
            (
                None,
                String::from("root"),
                (vec!["in".into()], vec!["out".into()]),
            ),
            (
                Some(()),
                String::from("A"),
                (vec!["in".into()], vec!["out".into()]),
            ),
        ];
        let edges = vec![((0, "out".into()), (1, "in".into()))];
        (nodes, edges)
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_cycles() -> Result<()> {
        let err = GraphBuilder::default()
            .set_graph_input(["in"])
            .set_graph_output(["out"])
            .node("A", (), ["in1", "in2"], ["out"])
            .node("B", (), ["in"], ["out"])
            .connect_input("in", ("A", "in1"))?
            .connect(("A", "out"), ("B", "in"))?
            .connect(("B", "out"), ("A", "in2"))?
            .build();

        let Err(e) = err else {
            panic!("expected error.");
        };

        assert_eq!(
            e,
            CommonGraphError::InvalidCycleDetected(
                ConnectionDetails {
                    source_node: "B".into(),
                    source_port: "out".into(),
                    target_node: "A".into(),
                    target_port: "in2".into(),
                }
                .into()
            )
        );
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_invalid_root() -> Result<()> {
        let root = (
            None,
            String::from("root"),
            (vec!["in".into()], vec!["out".into()]),
        );
        let a = (
            Some(()),
            String::from("A"),
            (vec!["in".into()], vec!["out".into()]),
        );
        let b = (
            Some(()),
            String::from("B"),
            (vec!["in".into()], vec!["out".into()]),
        );

        {
            let nodes = [a.clone(), b.clone()];
            let Err(e) = Graph::new(nodes, []) else {
                panic!("expected error.");
            };
            assert_eq!(
                e,
                CommonGraphError::InvalidRoot,
                "fails when there are no roots"
            );
        }
        {
            let nodes = [a.clone(), root.clone()];
            let Err(e) = Graph::new(nodes, []) else {
                panic!("expected error.");
            };
            assert_eq!(
                e,
                CommonGraphError::InvalidRoot,
                "fails when element 0 is not root"
            );
        }
        {
            let mut root2 = root.clone();
            root2.1 = "root2".into();
            let nodes = [root.clone(), a.clone(), root2.clone()];
            let Err(e) = Graph::new(nodes, []) else {
                panic!("expected error.");
            };
            assert_eq!(
                e,
                CommonGraphError::InvalidRoot,
                "fails when an element other than 0 is root"
            );
        }
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_empty_graph() -> Result<()> {
        let nodes: Vec<(Option<()>, String, (Vec<String>, Vec<String>))> = vec![(
            None,
            String::from("root"),
            (vec!["in".into()], vec!["out".into()]),
        )];

        let Err(e) = Graph::new(nodes, []) else {
            panic!("expected error");
        };
        assert_eq!(
            e,
            CommonGraphError::EmptyGraph,
            "fails when there is only a root node"
        );

        let nodes: Vec<(Option<()>, String, (Vec<String>, Vec<String>))> = vec![];
        let Err(e) = Graph::new(nodes, []) else {
            panic!("expected error");
        };
        assert_eq!(
            e,
            CommonGraphError::EmptyGraph,
            "fails when there are no nodes"
        );
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_missing_node() -> Result<()> {
        let (nodes, mut edges) = gen_graph_components();
        // Change incoming connection from index 1 to an invalid index 2
        edges[0].1 .0 = 2;

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(e, CommonGraphError::MissingNode(2));
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_missing_port() -> Result<()> {
        let (nodes, mut edges) = gen_graph_components();
        // Change incoming connection port to an invalid name
        edges[0].1 .1 = "not-real".into();

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(
            e,
            CommonGraphError::MissingPort(
                PortDetails {
                    port: "not-real".into(),
                    port_type: PortType::Input,
                    node: "A".into(),
                }
                .into()
            )
        );
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_orphaned_node() -> Result<()> {
        let (mut nodes, edges) = gen_graph_components();
        nodes.push((
            Some(()),
            String::from("B"),
            (vec!["in".into()], vec!["out".into()]),
        ));

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(e, CommonGraphError::OrphanedNode("B".into()));
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_duplicated_ports() -> Result<()> {
        let (mut nodes, edges) = gen_graph_components();
        // Set node A to have two outputs named "out"
        nodes[1].2 .1.push("out".into());

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(
            e,
            CommonGraphError::DuplicatePorts(
                PortDetails {
                    port_type: PortType::Output,
                    port: "out".into(),
                    node: "A".into()
                }
                .into()
            )
        );
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_duplicated_labels() -> Result<()> {
        let (mut nodes, edges) = gen_graph_components();
        let dupe = nodes[1].clone();
        nodes.push(dupe);

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(e, CommonGraphError::DuplicateLabels("A".into()));
        Ok(())
    }

    #[test]
    #[common_tracing]
    fn it_fails_on_fan_in_port() -> Result<()> {
        let (mut nodes, mut edges) = gen_graph_components();
        nodes.push((Some(()), "B".into(), (vec!["in".into()], vec![])));
        edges.push(((0, "out".into()), (2, "in".into())));
        edges.push(((1, "out".into()), (2, "in".into())));

        let Err(e) = Graph::new(nodes, edges) else {
            panic!("expected error");
        };
        assert_eq!(
            e,
            CommonGraphError::MultipleInputs(
                PortDetails {
                    node: "B".into(),
                    port: "in".into(),
                    port_type: PortType::Input,
                }
                .into()
            )
        );
        Ok(())
    }
}
