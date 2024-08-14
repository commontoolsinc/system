use thiserror::Error;

/// Result type for [CommonIfcError].
pub type Result<T> = ::core::result::Result<T, CommonIfcError>;

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
    #[error("Input '{0}' has insufficient permission to execute in this environment")]
    InvalidEnvironment(String),
    /// A graph provided did not pass structural validation.
    #[error("{0}")]
    InvalidGraph(String),
    /// A catch-all error.
    #[error("{0}")]
    InternalError(String),
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
