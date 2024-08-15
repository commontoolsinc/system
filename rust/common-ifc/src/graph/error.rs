use super::PortType;
use thiserror::Error;

/// Result type for [ValidationError].
pub type Result<T> = ::core::result::Result<T, GraphIntegrityError>;

/// Errors for graph integrity issues.
#[derive(PartialEq, Error, Debug)]
pub enum GraphIntegrityError {
    /// There were no nodes found in the graph.
    #[error("No nodes found in graph.")]
    EmptyGraph,

    /// There was a reference to a node not found in the graph.
    #[error("No node with id '{0}' found in graph.")]
    MissingNode(String),

    /// There was a reference to a port not found in a node.
    #[error("No {port_type} port '{port}' found in node '{node}'.")]
    MissingPort {
        /// Id of node.
        node: String,
        /// Type of port.
        port_type: PortType,
        /// Name of port.
        port: String,
    },

    /// There was two or more nodes with the same ID.
    #[error("Multiple nodes with id '{0}' found in graph.")]
    DuplicateNodeId(String),

    /// There was a node that is not connected to any other nodes in the graph.
    #[error("Node '{0}' is not connected to any other nodes.")]
    OrphanedNode(String),

    /// There was two or more ports of the same type found in node.
    #[error("Node '{node}' has multiple {port_type} ports with name '{port}'.")]
    DuplicatePorts {
        /// Id of node.
        node: String,
        /// Type of port.
        port_type: PortType,
        /// Name of port.
        port: String,
    },

    /// Node port has multiple incoming connections.
    #[error("Node '{node}' input port '{port}' has multiple incoming connections.")]
    MultipleInputs {
        /// Id of node.
        node: String,
        /// Name of port.
        port: String,
    },

    /// An internal, catch-all error.
    #[error("{0}")]
    InternalError(String),
}
