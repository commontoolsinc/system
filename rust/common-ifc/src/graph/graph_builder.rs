use std::{collections::HashMap, ops::Deref};

const NO_NODE_FOUND: &str = "No node found.";

pub struct Node<T> {
    inner: T,
    id: PortGraphId,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> Node<T> {
    pub fn id(&self) -> &PortGraphId {
        &self.id
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

/*
impl<'a, I, O, S> From<(I, O)> for Node<'a> where
    I: IntoIterator<Item = S>, O: IntoIterator<Item = S>, S: Into<String>
 {
    fn from(value: (I, O)) -> Self {
        let mut node = Node::default();
        let ref_node = &node;
        let mut inputs = value.0.into_iter().map(|s| (s, ref_node).into()).collect();
        let mut outputs = value.1.into_iter().map(|s| (s, ref_node).into()).collect();

        node.inputs.append(&mut inputs);
        node.outputs.append(&mut outputs);
        node
    }
}
*/

pub struct Connection<'a, T> {
    source: &'a Node<T>,
    target: &'a Node<T>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum PortGraphId {
    Root,
    Node(String),
}

impl From<String> for PortGraphId {
    fn from(value: String) -> Self {
        PortGraphId::Node(value)
    }
}

impl From<&str> for PortGraphId {
    fn from(value: &str) -> Self {
        String::from(value).into()
    }
}

#[derive(Default)]
struct PortGraph<'a, T> {
    nodes: HashMap<PortGraphId, Node<T>>,
    edges: Vec<Connection<'a, T>>,
}

impl<'a, T> PortGraph<'a, T> {
    pub fn add_node<P, I, O, S1, S2>(&mut self, id: P, inner: T, inputs: I, outputs: O)
    where
        P: Into<PortGraphId>,
        I: IntoIterator<Item = S1>,
        O: IntoIterator<Item = S2>,
        S1: Into<String>,
        S2: Into<String>,
    {
        let id = id.into();
        let node = Node {
            inputs: inputs.into_iter().map(|s| s.into()).collect(),
            outputs: outputs.into_iter().map(|s| s.into()).collect(),
            id: id.clone(),
            inner,
        };

        self.nodes.insert(id, node);
    }

    pub fn get_node<P: Into<PortGraphId>>(&self, id: P) -> Option<&Node<T>> {
        self.nodes.get(&id.into())
    }

    pub fn connect<P1, P2>(
        &'a mut self,
        source_id: P1,
        output_port: &str,
        target_id: P2,
        input_port: &str,
    ) -> std::result::Result<(), String>
    where
        P1: Into<PortGraphId>,
        P2: Into<PortGraphId>,
    {
        let source = self.nodes.get(&source_id.into()).ok_or(NO_NODE_FOUND)?;
        let target = self.nodes.get(&target_id.into()).ok_or(NO_NODE_FOUND)?;
        let connection = Connection { source, target };
        self.edges.push(connection);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;

    #[test]
    fn it_foo() {
        let mut graph = Graph::default();
        graph.node("foo".into(), vec!["in"], vec!["out"]);
        graph.node("foo2".into(), vec!["in"], vec!["out"]);
        graph.node("foo3".into(), vec!["in"], vec!["out"]);
        graph.connect("foo", "out", "foo2", "in").unwrap();
    }
}
