use crate::{graph::validation::validate_port_graph, Result};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

/// Keyable qualities for node IDs and port names.
pub trait PortGraphId: Hash + Eq + PartialEq + Display + Debug {}
impl<T> PortGraphId for T where T: Hash + Eq + PartialEq + Display + Debug {}

/// Type of port, either input or output.
pub enum PortType {
    /// Input-type ports.
    Input,
    /// Output-type ports.
    Output,
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
pub trait PortGraphNode<NodeId: PortGraphId, PortName: PortGraphId>: Debug {
    /// A unique to the graph identifier for this node.
    fn id(&self) -> &NodeId;
    /// All input ports for this node.
    fn inputs<'a>(&'a self) -> impl Iterator<Item = &'a PortName>
    where
        PortName: 'a;
    /// All output ports for this node.
    fn outputs<'a>(&'a self) -> impl Iterator<Item = &'a PortName>
    where
        PortName: 'a;
}

/// An edge in a [PortGraph].
pub trait PortGraphEdge<NodeId: PortGraphId, PortName: PortGraphId>: Debug {
    /// The source connection output port.
    fn source(&self) -> (&NodeId, &PortName);
    /// The target connection input port.
    fn target(&self) -> (&NodeId, &PortName);
}

impl<NodeId, PortName> PortGraphEdge<NodeId, PortName> for ((NodeId, PortName), (NodeId, PortName))
where
    NodeId: PortGraphId,
    PortName: PortGraphId,
{
    fn source(&self) -> (&NodeId, &PortName) {
        (&self.0 .0, &self.0 .1)
    }
    fn target(&self) -> (&NodeId, &PortName) {
        (&self.1 .0, &self.1 .1)
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
    type Node: PortGraphNode<Self::NodeId, Self::PortName>;

    /// Graph's edge type.
    type Edge: PortGraphEdge<Self::NodeId, Self::PortName>;

    /// The type of node's unique identifier.
    type NodeId: PortGraphId;

    /// The type of port names.
    type PortName: PortGraphId;

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
    fn validate_port_graph(&self) -> Result<()> {
        validate_port_graph(self)
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
