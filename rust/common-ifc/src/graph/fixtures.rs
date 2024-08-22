use crate::graph::{PortGraph, PortGraphNode};

/// [PortGraphNode] type for [TestGraph].
#[derive(Debug)]
pub struct TestNode {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl From<(String, Vec<String>, Vec<String>)> for TestNode {
    fn from(value: (String, Vec<String>, Vec<String>)) -> Self {
        TestNode {
            name: value.0,
            inputs: value.1,
            outputs: value.2,
        }
    }
}

impl PortGraphNode<String, String> for TestNode {
    fn id(&self) -> &String {
        &self.name
    }
    fn inputs<'a>(&'a self) -> impl Iterator<Item = &'a String>
    where
        String: 'a,
    {
        self.inputs.iter()
    }
    fn outputs<'a>(&'a self) -> impl Iterator<Item = &'a String>
    where
        String: 'a,
    {
        self.outputs.iter()
    }
}

pub type TestEdge = ((String, String), (String, String));

/// [PortGraph] type for tests.
#[derive(Debug)]
pub struct TestGraph {
    pub nodes: Vec<TestNode>,
    pub edges: Vec<TestEdge>,
}

impl PortGraph for TestGraph {
    type Node = TestNode;
    type Edge = TestEdge;
    type PortName = String;
    type NodeId = String;
    fn nodes(&self) -> impl Iterator<Item = &Self::Node> {
        self.nodes.iter()
    }
    fn edges(&self) -> impl Iterator<Item = &Self::Edge> {
        self.edges.iter()
    }
    fn get_node(&self, id: &Self::NodeId) -> Option<&Self::Node> {
        self.nodes.iter().find(|node| &node.name == id)
    }
}

impl TestGraph {
    /// Construct a [TestGraph] from structures easy-to-express
    /// in tests.
    pub fn from_iters<N, E, P>(nodes: N, edges: E) -> TestGraph
    where
        N: IntoIterator<Item = (&'static str, P, P)>,
        E: IntoIterator<Item = ((&'static str, &'static str), (&'static str, &'static str))>,
        P: IntoIterator<Item = &'static str>,
    {
        let mut out_nodes = vec![];
        let mut out_edges = vec![];
        for node in nodes.into_iter() {
            out_nodes.push(TestNode {
                name: node.0.into(),
                inputs: node.1.into_iter().map(String::from).collect(),
                outputs: node.2.into_iter().map(String::from).collect(),
            });
        }
        for edge in edges.into_iter() {
            out_edges.push((
                (edge.0 .0.into(), edge.0 .1.into()),
                (edge.1 .0.into(), edge.1 .1.into()),
            ));
        }
        TestGraph {
            nodes: out_nodes,
            edges: out_edges,
        }
    }
}
