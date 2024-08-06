use thiserror::Error;

/// Result type for [IfcError].
pub type Result<T> = ::core::result::Result<T, IfcError>;

/// Errors for policy validation and other errors.
#[derive(PartialEq, Error, Debug)]
pub enum IfcError {
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
}
