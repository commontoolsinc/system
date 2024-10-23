use std::fmt::Debug;
use thiserror::Error;

/// [`std::result::Result`] type with [Error]
/// as its error.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Various errors that may be encountered when invoking runtime code.
#[derive(Error, PartialEq, Debug)]
pub enum Error {
    /// An unsupported VM was requested.
    #[error("Requested VM is unsupported.")]
    UnsupportedVm,

    /// An error occurred during instantiation.
    #[error("Failed to instantiate a sandbox: {0}")]
    InstantiationFailure(String),

    /// An error occurred during linking.
    #[error("Failed during linking: {0}")]
    LinkerFailure(String),

    /// An error occurred while executing a VM function.
    #[error("Failed to invoke sandbox: {0}")]
    InvocationFailure(String),

    /// An unexpected internal error occurred
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::InternalError(value)
    }
}
