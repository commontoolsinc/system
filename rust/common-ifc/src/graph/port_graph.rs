use crate::graph::{integrity::check_integrity, Result};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Keyable qualities for node IDs and port names.
pub trait PortGraphId: Hash + Eq + PartialEq + Display + Debug {}
impl<T> PortGraphId for T where T: Hash + Eq + PartialEq + Display + Debug {}

/// Type of port, either input or output.
#[derive(Debug, PartialEq)]
pub enum PortType {
    /// Input-type ports.
    Input,
    /// Output-type ports.
    Output,
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PortType {
    /// Return a string slice name for this [PortType].
    pub fn as_str(&self) -> &'static str {
        match self {
            PortType::Input => "input",
            PortType::Output => "output",
        }
    }
}

/// A node in a [PortGraph].
pub trait PortGraphNode<NodeId: PortGraphId>: Debug {
    /// A unique to the graph identifier for this node.
    fn id(&self) -> &NodeId;
    /// All input ports for this node.
    fn inputs(&self) -> impl Iterator<Item = &str>;
    /// All output ports for this node.
    fn outputs(&self) -> impl Iterator<Item = &str>;
    /// Whether this node is a root node. Root nodes are both the entry
    /// points for an execution graph, as well as the only valid path
    /// for a cycle.
    fn is_root(&self) -> bool;
}

/// An edge in a [PortGraph].
pub trait PortGraphEdge<NodeId: PortGraphId>: Debug {
    /// The source connection output port.
    fn source(&self) -> (&NodeId, &str);
    /// The target connection input port.
    fn target(&self) -> (&NodeId, &str);
}

impl<NodeId, S> PortGraphEdge<NodeId> for ((NodeId, S), (NodeId, S))
where
    NodeId: PortGraphId,
    S: AsRef<str> + Debug,
{
    fn source(&self) -> (&NodeId, &str) {
        (&self.0 .0, self.0 .1.as_ref())
    }
    fn target(&self) -> (&NodeId, &str) {
        (&self.1 .0, self.1 .1.as_ref())
    }
}

/// Trait implementing a Port Graph, a directed graph consisting
/// of vertices that have named input and output ports,
/// with edges represented by a directed connection from
/// one vertex's port to another.
///
/// See [PortGraph::validate_port_graph] for expected constraints.
pub trait PortGraph: Debug {
    /// Graph's node type.
    type Node: PortGraphNode<Self::NodeId>;

    /// Graph's edge type.
    type Edge: PortGraphEdge<Self::NodeId>;

    /// The type of node's unique identifier.
    type NodeId: PortGraphId;

    /// Returns the root of the port graph, if any.
    fn root(&self) -> Option<&Self::Node>;

    /// Return all nodes in the graph.
    fn nodes(&self) -> impl Iterator<Item = &Self::Node>;

    /// Return all edges in the graph.
    fn edges(&self) -> impl Iterator<Item = &Self::Edge>;

    /// Returns a node with `id` in the graph if found.
    fn get_node(&self, id: &Self::NodeId) -> Option<&Self::Node>;

    /// Validates the structure of the graph, ensuring
    /// the following properties:
    ///
    /// * Node ids are unique across all nodes
    /// * Port ids are unique across ports of the
    ///   same type in a node (two output ports can't have the same id,
    ///   but an input and output port can).
    /// * No unconnected nodes.
    /// * Edges must contain node and port references
    ///   that exist in the collection of nodes.
    /// * No input ports with multiple edges (no fan-in).
    /// * Edges must be unique across the graph. Implicitly
    ///   enforced by not allowing fan-in.
    fn check_integrity(&self) -> Result<()> {
        check_integrity(self)
    }

    /// Returns the root node, if any.
    fn get_root(&self) -> Option<&Self::Node> {
        self.nodes().find(|&node| node.is_root())
    }

    /// Get all connections for this node's ports. If
    /// a `port_type` provided, only connections containing
    /// that type of node port are returned.
    fn get_connections<'a>(
        &'a self,
        node: &'a Self::Node,
        port_type: Option<PortType>,
    ) -> impl Iterator<Item = &'a Self::Edge> {
        let id = node.id();
        self.edges().filter(move |edge| {
            let source = edge.source();
            let target = edge.target();
            match port_type {
                Some(PortType::Input) => target.0 == id,
                Some(PortType::Output) => source.0 == id,
                None => source.0 == id || target.0 == id,
            }
        })
    }
}
