use super::{CommonGraphError, Edge, Graph, Result};
use std::fmt::Debug;

type BuilderNode<T> = (String, usize, T, (Vec<String>, Vec<String>));

/// Builder utility for [`Graph`].
#[derive(Clone, Debug)]
pub struct GraphBuilder<T: Debug> {
    nodes: Vec<BuilderNode<T>>,
    edges: Vec<Edge>,
    inputs: Option<Vec<String>>,
    outputs: Option<Vec<String>>,
    root_label: Option<String>,
}

impl<T> GraphBuilder<T>
where
    T: Debug,
{
    /// Set the label of the graph/root node.
    pub fn set_label<S: Into<String>>(mut self, label: S) -> Self {
        self.root_label = Some(label.into());
        self
    }

    /// Set graph inputs/root node outputs.
    pub fn set_graph_input<I, S>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inputs = Some(serialize_io(inputs));
        self
    }

    /// Set graph outputs/root node inputs.
    pub fn set_graph_output<I, S>(mut self, outputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.outputs = Some(serialize_io(outputs));
        self
    }

    /// Add a new node.
    pub fn node<I, O, S, S1, S2>(mut self, id: S, inner: T, inputs: I, outputs: O) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S1>,
        O: IntoIterator<Item = S2>,
        S1: Into<String>,
        S2: Into<String>,
    {
        // Offset the indices by 1 so that the root node
        // can be set at the 0th position.
        let index = self.nodes.len() + 1;

        self.nodes.push((
            id.into(),
            index,
            inner,
            (serialize_io(inputs), serialize_io(outputs)),
        ));
        self
    }

    /// Connect graph input to a node port.
    pub fn connect_input<S1, S2, S3>(mut self, from_port: S1, to_port: (S2, S3)) -> Result<Self>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let node_name = to_port.0.into();
        let (_, node_index, _, _) = self.get_node(&node_name)?;

        let edge: Edge = (
            (0, from_port.into()),
            (node_index.to_owned(), to_port.1.into()),
        );
        self.edges.push(edge);
        Ok(self)
    }

    /// Connect a node port to the graph output.
    pub fn connect_output<S1, S2, S3>(mut self, from_port: (S1, S2), to_port: S3) -> Result<Self>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        let node_name = from_port.0.into();
        let (_, node_index, _, _) = self.get_node(&node_name)?;

        let edge: Edge = (
            (node_index.to_owned(), from_port.1.into()),
            (0, to_port.into()),
        );
        self.edges.push(edge);
        Ok(self)
    }

    /// Connect one node port to another.
    pub fn connect<S1, S2, S3, S4>(mut self, from_port: (S1, S2), to_port: (S3, S4)) -> Result<Self>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        let (from_index, to_index) = {
            let from_name = from_port.0.into();
            let (_, from_index, _, _) = self.get_node(&from_name)?;
            let to_name = to_port.0.into();
            let (_, to_index, _, _) = self.get_node(&to_name)?;
            (from_index, to_index)
        };

        let edge: Edge = (
            (from_index.to_owned(), from_port.1.into()),
            (to_index.to_owned(), to_port.1.into()),
        );
        self.edges.push(edge);
        Ok(self)
    }

    fn get_node(&self, label: &str) -> Result<&BuilderNode<T>> {
        self.nodes
            .iter()
            .find(|n| n.0.as_str() == label)
            .ok_or_else(|| {
                CommonGraphError::GraphBuilderFailure(format!("No node with name {}.", label))
            })
    }

    /// Builds a [Graph].
    pub fn build(self) -> Result<Graph<T>> {
        // Note that "inputs" and "outputs" to the graph
        // are represented by the root node, where graph "inputs"
        // and "outputs" are node outputs and inputs (reversed).
        let inputs = self.inputs.unwrap_or_default();
        let outputs = self.outputs.unwrap_or_default();
        let root_label = self.root_label.unwrap_or_else(|| "Root".into());

        // Map the helper HashMap to an indexed vec.
        // The index references are 1-index based, so we can
        // safely prepend a "root" node at 0.
        let mut nodes = Vec::with_capacity(self.nodes.len() + 1);
        nodes.push((None, root_label, (outputs, inputs)));
        for (label, _, node, io) in self.nodes.into_iter() {
            nodes.push((Some(node), label, io));
        }
        Graph::new(nodes, self.edges)
    }
}

impl<T> std::default::Default for GraphBuilder<T>
where
    T: Debug,
{
    fn default() -> Self {
        GraphBuilder {
            nodes: vec![],
            edges: vec![],
            inputs: None,
            outputs: None,
            root_label: None,
        }
    }
}

/// Genericize taking an iterator of a string-like.
fn serialize_io<I, S>(def: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    def.into_iter().map(|s| s.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(s: &str) -> String {
        s.into()
    }

    #[test]
    fn it_builds_a_graph() -> Result<()> {
        let builder = GraphBuilder::default();
        let graph = builder
            .set_label("RootNode")
            .set_graph_input(vec!["in1", "in2"])
            .set_graph_output(vec!["out1", "out2"])
            .node("A", (), vec!["A-in"], vec!["A-out"])
            .node("B", (), vec!["B-in1", "B-in2"], vec!["B-out"])
            .connect_input("in1", ("A", "A-in"))?
            .connect_input("in1", ("B", "B-in1"))?
            .connect(("A", "A-out"), ("B", "B-in2"))?
            .connect_output(("B", "B-out"), "out1")?
            .build()?;
        let nodes = graph.nodes();
        let root = &nodes[0];
        let a = &nodes[1];
        let b = &nodes[2];

        assert_eq!(root.label(), "RootNode");
        assert_eq!(
            root.inputs(),
            &[(s("out1"), Some((2, s("B-out")))), (s("out2"), None)]
        );
        assert_eq!(
            root.outputs(),
            &[
                (s("in1"), Some(vec![(1, s("A-in")), (2, s("B-in1"))])),
                (s("in2"), None)
            ]
        );

        assert_eq!(a.inputs(), &[(s("A-in"), Some((0, s("in1"))))],);
        assert_eq!(a.outputs(), &[(s("A-out"), Some(vec![(2, s("B-in2"))])),]);

        assert_eq!(
            b.inputs(),
            &[
                (s("B-in1"), Some((0, s("in1")))),
                (s("B-in2"), Some((1, s("A-out")))),
            ],
        );
        assert_eq!(b.outputs(), &[(s("B-out"), Some(vec![(0, s("out1"))]))]);
        Ok(())
    }

    #[test]
    fn it_fails_for_unknown_nodes() {
        let builder = GraphBuilder::default()
            .set_graph_input(vec!["in"])
            .set_graph_output(vec!["out"])
            .node("A", (), vec!["A-in"], vec!["A-out"])
            .node("B", (), vec!["B-in"], vec!["B-out"]);

        assert!(builder
            .clone()
            .connect_input("in", ("Unknown", "A-in"))
            .is_err());
        assert!(builder
            .clone()
            .connect_output(("Unknown", "A-out"), "out")
            .is_err());
        assert!(builder
            .clone()
            .connect(("A", "A-out"), ("Unknown", "U-in"))
            .is_err());
    }
}
