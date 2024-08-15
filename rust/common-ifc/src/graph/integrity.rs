use crate::graph::{
    GraphIntegrityError, PortGraph, PortGraphEdge, PortGraphId, PortGraphNode, PortType, Result,
};
use std::collections::HashSet;

/// Validates a [PortGraph]'s integrity.
/// See [PortGraph::check_integrity] for details.
pub fn check_integrity<G: PortGraph + ?Sized>(graph: &G) -> Result<()> {
    let mut node_set: HashSet<&G::NodeId> = HashSet::default();
    let mut has_nodes = false;
    let nodes = graph.nodes();
    for node in nodes {
        has_nodes = true;
        let id = node.id();

        // Check unique node ids
        if !node_set.insert(id) {
            return Err(GraphIntegrityError::DuplicateNodeId(id.to_string()));
        }

        validate_node_ports::<G>(node)?;
        validate_node_connections::<G>(graph, node)?;
    }

    if !has_nodes {
        Err(GraphIntegrityError::EmptyGraph)
    } else {
        Ok(())
    }
}

/// Validates a given `node` that all port names are unique
/// amongst their port types.
fn validate_node_ports<G: PortGraph + ?Sized>(node: &G::Node) -> Result<()> {
    let id = node.id();

    check_port_uniqueness(node.inputs(), id, PortType::Input)?;
    check_port_uniqueness(node.outputs(), id, PortType::Output)?;

    fn check_port_uniqueness<'a, NodeId, T>(
        ports: T,
        node_id: NodeId,
        port_type: PortType,
    ) -> Result<()>
    where
        NodeId: PortGraphId,
        T: Iterator<Item = &'a str>,
    {
        let mut port_set: HashSet<&str> = HashSet::default();
        for port in ports {
            if !port_set.insert(port) {
                return Err(GraphIntegrityError::DuplicatePorts {
                    port: port.into(),
                    node: node_id.to_string(),
                    port_type,
                });
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
            return Err(GraphIntegrityError::MultipleInputs {
                node: id.to_string(),
                port: target.1.into(),
            });
        }
    }

    if !has_connections {
        Err(GraphIntegrityError::OrphanedNode(id.to_string()))
    } else {
        Ok(())
    }
}

/// Validates the existence of a node with given `node_id`
/// in the hash map with `port_type` port named `port_name`.
fn validate_port_exists<G: PortGraph + ?Sized>(
    graph: &G,
    node_id: &G::NodeId,
    port_name: &str,
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
        return Err(GraphIntegrityError::MissingNode(node_id.to_string()));
    }

    Err(GraphIntegrityError::MissingPort {
        node: node_id.to_string(),
        port_type,
        port: port_name.into(),
    })
}

#[cfg(test)]
mod tests {
    use crate::graph::{fixtures::TestGraph, GraphIntegrityError, PortGraph, PortType, Result};

    fn gen_graph() -> TestGraph {
        TestGraph::from_iters(
            [
                ("A", vec![], vec!["a_out_1", "a_out_2"]),
                ("B", vec!["b_in"], vec!["b_out"]),
                ("C", vec!["c_in"], vec!["c_out"]),
                ("D", vec!["d_in_1", "d_in_2", "d_in_3"], vec!["d_out"]),
            ],
            [
                (("A", "a_out_1"), ("B", "b_in")),
                (("A", "a_out_2"), ("C", "c_in")),
                (("B", "b_out"), ("D", "d_in_1")),
                (("C", "c_out"), ("D", "d_in_2")),
            ],
        )
    }

    #[test]
    fn it_validates_graph() -> Result<()> {
        let graph = gen_graph();
        graph.check_integrity()?;
        Ok(())
    }

    #[test]
    fn it_fails_with_empty_graph() {
        let nodes: [(&'static str, Vec<&'static str>, Vec<&'static str>); 0] = [];
        let edges: [((&'static str, &'static str), (&'static str, &'static str)); 0] = [];
        let graph = TestGraph::from_iters(nodes, edges);
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::EmptyGraph)
        );
    }

    #[test]
    fn it_fails_with_orphaned_nodes() {
        let mut graph = gen_graph();
        graph
            .nodes
            .push(("E".into(), vec!["e_in".into()], vec![]).into());
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::OrphanedNode("E".into()))
        );
    }

    #[test]
    fn it_fails_with_duplicate_node_ids() {
        let mut graph = gen_graph();
        graph
            .nodes
            .push(("B".into(), vec!["b_in".into()], vec![]).into());
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::DuplicateNodeId("B".into()))
        );
    }

    #[test]
    fn it_fails_with_duplicate_input_port_names() {
        let mut graph = gen_graph();
        graph
            .nodes
            .push(("E".into(), vec!["e_in".into(), "e_in".into()], vec![]).into());
        graph
            .edges
            .push((("B".into(), "b_out".into()), ("E".into(), "e_in".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::DuplicatePorts {
                node: "E".into(),
                port_type: PortType::Input,
                port: "e_in".into(),
            })
        );
    }

    #[test]
    fn it_fails_with_duplicate_output_port_names() {
        let mut graph = gen_graph();
        graph.nodes.push(
            (
                "E".into(),
                vec!["e_in".into()],
                vec!["e_out".into(), "e_out".into()],
            )
                .into(),
        );
        graph
            .edges
            .push((("B".into(), "b_out".into()), ("E".into(), "e_in".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::DuplicatePorts {
                node: "E".into(),
                port_type: PortType::Output,
                port: "e_out".into(),
            })
        );
    }

    #[test]
    fn it_fails_with_missing_input_node_id_references() {
        let mut graph = gen_graph();
        graph
            .edges
            .push((("C".into(), "c_out".into()), ("Foo".into(), "nope".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::MissingNode("Foo".into()))
        );
    }

    #[test]
    fn it_fails_with_missing_output_node_id_references() {
        let mut graph = gen_graph();
        graph.edges.push((
            ("Foo".into(), "f_out".into()),
            ("D".into(), "d_in_3".into()),
        ));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::MissingNode("Foo".into()))
        );
    }

    #[test]
    fn it_fails_with_missing_input_port_references() {
        let mut graph = gen_graph();
        graph
            .edges
            .push((("C".into(), "c_out".into()), ("D".into(), "nope".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::MissingPort {
                node: "D".into(),
                port_type: PortType::Input,
                port: "nope".into(),
            })
        );
    }

    #[test]
    fn it_fails_with_missing_output_port_references() {
        let mut graph = gen_graph();
        graph
            .edges
            .push((("B".into(), "nope".into()), ("D".into(), "d_in_3".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::MissingPort {
                node: "B".into(),
                port_type: PortType::Output,
                port: "nope".into(),
            })
        );
    }

    #[test]
    fn it_fails_with_fan_in() {
        let mut graph = gen_graph();
        graph
            .nodes
            .push(("E".into(), vec!["e_in".into()], vec![]).into());
        graph
            .edges
            .push((("B".into(), "b_out".into()), ("E".into(), "e_in".into())));
        graph
            .edges
            .push((("D".into(), "d_out".into()), ("E".into(), "e_in".into())));
        assert_eq!(
            graph.check_integrity(),
            Err(GraphIntegrityError::MultipleInputs {
                node: "E".into(),
                port: "e_in".into()
            })
        );
    }
}
