use crate::{
    graph::{PortGraph, PortGraphEdge, PortGraphId, PortGraphNode, PortType},
    CommonIfcError, Result,
};
use std::collections::HashSet;

/// Validates a [PortGraph]. See [PortGraph::validate_port_graph] for details.
pub fn validate_port_graph<G: PortGraph + ?Sized>(graph: &G) -> Result<()> {
    let mut node_set: HashSet<&G::NodeId> = HashSet::default();
    let nodes = graph.nodes();
    for node in nodes {
        let id = node.id();

        // Check unique node ids
        if !node_set.insert(id) {
            return Err(CommonIfcError::InvalidGraph(format!(
                "Multiple nodes with id '{}' found in graph.",
                id
            )));
        }

        validate_node_ports::<G>(node)?;
        validate_node_connections::<G>(graph, node)?;
    }
    Ok(())
}

/// Validates a given `node` that all port names are unique
/// amongst their port types.
fn validate_node_ports<G: PortGraph + ?Sized>(node: &G::Node) -> Result<()> {
    let id = node.id();

    check_port_uniqueness(node.inputs(), id, PortType::Input)?;
    check_port_uniqueness(node.outputs(), id, PortType::Output)?;

    fn check_port_uniqueness<'a, PortName, NodeId, T>(
        ports: T,
        node_id: NodeId,
        port_type: PortType,
    ) -> Result<()>
    where
        PortName: PortGraphId + 'a,
        NodeId: PortGraphId,
        T: Iterator<Item = &'a PortName>,
    {
        let mut port_set: HashSet<&PortName> = HashSet::default();
        for port in ports {
            if !port_set.insert(port) {
                return Err(CommonIfcError::InvalidGraph(format!(
                    "Multiple {} ports with name '{}' in node '{}'.",
                    port_type.as_str(),
                    port,
                    node_id
                )));
            }
        }
        Ok(())
    }

    Ok(())
}

/// Validates the connections of `node`, that it is connected
/// to another node in the graph, an input port doesn't have
/// multiple incoming connections, and all edges with this node are valid.
fn validate_node_connections<G: PortGraph + ?Sized>(graph: &G, node: &G::Node) -> Result<()> {
    let id = node.id();
    let mut has_connections = false;
    let mut incoming_set = HashSet::new();

    for edge in graph.get_connections(node, None) {
        has_connections = true;
        let source = edge.source();
        let target = edge.target();
        validate_port_exists::<G>(graph, source.0, source.1, PortType::Output)?;
        validate_port_exists::<G>(graph, target.0, target.1, PortType::Input)?;

        if target.0 == id && !incoming_set.insert(target.1) {
            return Err(CommonIfcError::InvalidGraph(format!(
                "Input '{}' on node '{}' has multiple incoming connections.",
                target.1, id
            )));
        }
    }

    if !has_connections {
        Err(CommonIfcError::InvalidGraph(format!(
            "Node '{}' is not connected to any other nodes.",
            id
        )))
    } else {
        Ok(())
    }
}

/// Validates the existence of a node with given `node_id`
/// in the hash map with `port_type` port named `port_name`.
fn validate_port_exists<G: PortGraph + ?Sized>(
    graph: &G,
    node_id: &G::NodeId,
    port_name: &G::PortName,
    port_type: PortType,
) -> Result<()> {
    if let Some(node) = graph.get_node(node_id) {
        if match port_type {
            PortType::Input => node.inputs().any(|p| p == port_name),
            PortType::Output => node.outputs().any(|p| p == port_name),
        } {
            return Ok(());
        }
    } else {
        return Err(CommonIfcError::InvalidGraph(format!(
            "Node '{}' is not in the graph.",
            node_id,
        )));
    }

    let port_type_name = match port_type {
        PortType::Input => "input",
        PortType::Output => "output",
    };
    Err(CommonIfcError::InvalidGraph(format!(
        "Node '{}' does not contain an {} port with the name '{}'.",
        node_id, port_type_name, port_name,
    )))
}

#[cfg(test)]
mod tests {
    use super::PortGraph;
    use crate::graph::TestGraph;

    #[test]
    fn it_validates_graph() -> crate::Result<()> {
        let mut graph = TestGraph::from_iters(
            [
                ("A", vec![], vec!["a_out_1", "a_out_2"]),
                ("B", vec!["b_in"], vec!["b_out"]),
                ("C", vec!["c_in"], vec!["c_out"]),
                ("D", vec!["d_in_1", "d_in_2", "d_in_3"], vec![]),
            ],
            [
                (("A", "a_out_1"), ("B", "b_in")),
                (("A", "a_out_2"), ("C", "c_in")),
                (("B", "b_out"), ("D", "d_in_1")),
                (("C", "c_out"), ("D", "d_in_2")),
            ],
        );
        graph.validate_port_graph()?;

        // Unconnected nodes should fail.
        graph
            .nodes
            .push(("E".into(), vec!["e_in".into()], vec![]).into());
        assert!(graph.validate_port_graph().is_err());

        // Connecting should pass, allowing fan-out
        graph
            .edges
            .push((("C".into(), "c_out".into()), ("E".into(), "e_in".into())));
        graph.validate_port_graph()?;

        // Should not allow fan-in
        {
            graph
                .edges
                .push((("B".into(), "b_out".into()), ("E".into(), "e_in".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            graph.validate_port_graph()?;
        }

        // Node ids must be unique.
        graph
            .nodes
            .push(("E".into(), vec!["e_in".into()], vec![]).into());
        assert!(graph.validate_port_graph().is_err());
        let _ = graph.nodes.pop();

        // Node ports must be unique among their port types
        {
            {
                let e_node = graph.nodes.last_mut().unwrap();
                e_node.inputs = vec!["e_in".into(), "e_in".into()];
            }
            assert!(graph.validate_port_graph().is_err());
            {
                let e_node = graph.nodes.last_mut().unwrap();
                e_node.inputs = vec!["e_in".into()];
            }
            graph.validate_port_graph()?;
        }

        // Edges must contain references to nodes and ports found in the graph.
        {
            graph
                .edges
                .push((("C".into(), "c_out".into()), ("Foo".into(), "nope".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            graph
                .edges
                .push((("C".into(), "c_out".into()), ("D".into(), "nope".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            graph
                .edges
                .push((("Foo".into(), "nope".into()), ("D".into(), "d_in_3".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            graph
                .edges
                .push((("B".into(), "nope".into()), ("D".into(), "d_in_3".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            // Also checks validity when connecting an input to an output
            graph
                .edges
                .push((("B".into(), "b_in".into()), ("C".into(), "c_out".into())));
            assert!(graph.validate_port_graph().is_err());
            let _ = graph.edges.pop();
            graph.validate_port_graph()?;
        }

        Ok(())
    }
}
