use crate::LabelType;
use common_graph::CommonGraphError;
use thiserror::Error;

/// Result type for [`CommonIfcError`].
pub type Result<T> = ::core::result::Result<T, CommonIfcError>;

/// Details of the origins of a [`CommonIfcError::PolicyViolation`].
#[derive(PartialEq, Debug)]
pub struct PolicyViolationSource {
    /// Underlying error causing a policy violation.
    pub cause: CommonIfcError,
    /// Input port that caused the violation.
    pub input: String,
    /// The label type of violation.
    pub label_type: LabelType,
    /// Optional source node in a graph of violation.
    pub node: Option<String>,
}

/// Errors for policy validation and other errors.
#[derive(PartialEq, Error, Debug)]
pub enum CommonIfcError {
    /// There was an error during deserializing or
    /// converting data.
    #[error("Conversion error")]
    Conversion,
    /// The policy is malformed or illegal.
    #[error("{0}")]
    InvalidPolicy(String),
    /// The environment is insufficient based on the policy.
    #[error("Insufficient permission to execute in this environment")]
    InvalidEnvironment,
    /// A policy violation occurred.
    /// [`CommonIfcError`] with additional graph context.
    #[error("Policy violation: {}", .0.cause)]
    PolicyViolation(Box<PolicyViolationSource>),
    /// There was an error in the underlying graph.
    #[error("{0}")]
    InvalidGraph(String),
    /// A strong assertion has failed.
    #[error("UNEXPECTED: {0}")]
    Unexpected(String),
    /// A catch-all error.
    #[error("{0}")]
    InternalError(String),
}

impl From<std::io::Error> for CommonIfcError {
    fn from(e: std::io::Error) -> Self {
        e.to_string().into()
    }
}

impl From<std::string::FromUtf8Error> for CommonIfcError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        e.to_string().into()
    }
}

impl From<String> for CommonIfcError {
    fn from(value: String) -> Self {
        CommonIfcError::InternalError(value)
    }
}

impl From<CommonGraphError> for CommonIfcError {
    fn from(e: CommonGraphError) -> Self {
        CommonIfcError::InvalidGraph(e.to_string())
    }
}

impl<T> From<T> for CommonIfcError
where
    T: Into<Box<PolicyViolationSource>>,
{
    fn from(value: T) -> Self {
        CommonIfcError::PolicyViolation(value.into())
    }
}
