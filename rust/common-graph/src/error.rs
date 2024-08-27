use crate::PortType;
use thiserror::Error;

#[cfg(doc)]
use crate::GraphBuilder;

/// Details of a port in a graph.
#[derive(PartialEq, Debug)]
pub struct PortDetails {
    /// Label of the node.
    pub node: String,
    /// Name of the port.
    pub port: String,
    /// Type of port.
    pub port_type: PortType,
}

/// Details of a graph connection.
#[derive(PartialEq, Debug)]
pub struct ConnectionDetails {
    /// Label of source node.
    pub source_node: String,
    /// Name of source port.
    pub source_port: String,
    /// Label of target node.
    pub target_node: String,
    /// Name of target port.
    pub target_port: String,
}

/// Result type for [CommonGraphError].
pub type Result<T> = ::core::result::Result<T, CommonGraphError>;

/// Errors for graph integrity issues.
#[derive(PartialEq, Error, Debug)]
pub enum CommonGraphError {
    /// There was an invalid cycle in the graph, backtracking
    /// to a node outside of the root node.
    #[error("An invalid cycle was detected in the graph.")]
    InvalidCycleDetected(Box<ConnectionDetails>),

    /// There was either no root nodes, multiple root nodes,
    /// or a root node not first in the definition.
    #[error("There must be a single root node in the 0th position.")]
    InvalidRoot,

    /// There were no nodes found in the graph.
    #[error("No nodes found in graph.")]
    EmptyGraph,

    /// There was a reference to a node not found in the graph.
    #[error("No node with index '{0}' found in graph.")]
    MissingNode(usize),

    /// There was a reference to a port not found in a node.
    #[error("No {} port '{}' found in node '{}'.", .0.port_type, .0.port, .0.node)]
    MissingPort(Box<PortDetails>),

    /// There was a node that is not connected to any other nodes in the graph.
    #[error("Node '{0}' is not connected to any other nodes.")]
    OrphanedNode(String),

    /// There was two or more nodes with the same label.
    #[error("Multiple nodes with the '{0}' label.")]
    DuplicateLabels(String),

    /// There was two or more ports of the same type found in node.
    #[error("Node '{}' has multiple {} ports with name '{}'.", .0.node, .0.port_type, .0.port)]
    DuplicatePorts(Box<PortDetails>),

    /// Node port has multiple incoming connections.
    #[error("Node '{}' input port '{}' has multiple incoming connections.", .0.node, .0.port)]
    MultipleInputs(Box<PortDetails>),

    /// A catch-all failure.
    #[error("{0}")]
    InternalError(String),

    /// A failure that should not occur.
    #[error("{0}")]
    Unexpected(String),

    /// Invalid configurations were provided to [GraphBuilder].
    #[error("{0}")]
    GraphBuilderFailure(String),
}

impl From<std::io::Error> for CommonGraphError {
    fn from(value: std::io::Error) -> Self {
        value.to_string().into()
    }
}

impl From<std::string::FromUtf8Error> for CommonGraphError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        value.to_string().into()
    }
}

impl From<String> for CommonGraphError {
    fn from(value: String) -> Self {
        CommonGraphError::InternalError(value)
    }
}

impl From<&str> for CommonGraphError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}
