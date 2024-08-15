use crate::{graph::GraphIntegrityError, LabelType};
use thiserror::Error;

/// Result type for [CommonIfcError].
pub type Result<T> = ::core::result::Result<T, CommonIfcError>;

/// Location in a graph of where an error occurred.
#[derive(Debug, PartialEq)]
pub struct ErrorSource {
    /// The source node where the error originated.
    pub source: String,
    /// The port of the source node where the error originated.
    pub source_port: String,
    /// The target node where the error originated.
    pub target: String,
    /// The port of the target node where the error originated.
    pub target_port: String,
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{} -> {}::{}",
            self.source, self.source_port, self.target, self.target_port
        )
    }
}

/// Errors for policy validation and other errors.
#[derive(PartialEq, Error, Debug)]
pub enum CommonIfcError {
    /// There was an error building a [Data<T>] from
    /// a [common_protos::common::LabeledData].
    #[error("Could not convert from protobuf into Data<T>.")]
    ConversionFailure,
    /// Policy was malformed.
    #[error("Policy is missing {label_type} level '{level}'.")]
    PolicyMissingDefinition {
        /// The label type.
        label_type: LabelType,
        /// The label level.
        level: String,
    },
    /// There was an error during deserializing or
    /// A graph provided did not pass structural validation.
    #[error("{0}")]
    GraphIntegrityError(GraphIntegrityError),
    /// Context failed validation.
    #[error("Context failed validating {label_type} label: {details}")]
    InvalidContext {
        /// The label type.
        label_type: LabelType,
        /// Additional details provided from the context.
        details: String,
    },
    /// There was a policy error in a graph.
    #[error("Validation failed for {label_type} connecting {error_source}: {details}")]
    ValidationError {
        /// Information of where in the graph the error occurred.
        error_source: Box<ErrorSource>,
        /// The label type.
        label_type: String,
        /// Additional details provided from the context.
        details: String,
    },
    /// A catch-all error.
    #[error("{0}")]
    InternalError(String),
}

impl From<GraphIntegrityError> for CommonIfcError {
    fn from(value: GraphIntegrityError) -> Self {
        CommonIfcError::GraphIntegrityError(value)
    }
}

impl From<std::io::Error> for CommonIfcError {
    fn from(e: std::io::Error) -> Self {
        CommonIfcError::InternalError(format!("{:#?}", e))
    }
}

impl From<std::string::FromUtf8Error> for CommonIfcError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        CommonIfcError::InternalError(format!("{:#?}", e))
    }
}

impl From<String> for CommonIfcError {
    fn from(value: String) -> Self {
        CommonIfcError::InternalError(value)
    }
}

impl From<&str> for CommonIfcError {
    fn from(value: &str) -> Self {
        String::from(value).into()
    }
}
