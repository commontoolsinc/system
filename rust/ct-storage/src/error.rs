use std::fmt::Debug;
use thiserror::Error;

/// [`std::result::Result`] type with [Error] as its error.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Errors occurring from this crate.
#[derive(Error, PartialEq, Debug)]
pub enum Error {
    /// There was an error during type conversion.
    #[error("Conversion error.")]
    Conversion,
    /// There was an error performing IO.
    #[error("IO error: {0}")]
    Io(String),
    /// There was an encoding error.
    #[error("Encoding error: {0}")]
    Encoding(String),
    /// There was an out of range request.
    #[error("Request out of range.")]
    OutOfRange,
    /// An error within tree operations.
    #[error("Tree error: {0}")]
    Tree(String),
    /// An error occurred.
    #[error("{0}")]
    Internal(String),
    /// An error occurred that should never happen.
    #[error("Unexpected operation.")]
    Unexpected,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(_: std::num::TryFromIntError) -> Self {
        Error::Conversion
    }
}

impl From<ranked_prolly_tree::Error> for Error {
    fn from(value: ranked_prolly_tree::Error) -> Self {
        Error::Tree(value.to_string())
    }
}

impl From<Error> for ranked_prolly_tree::Error {
    fn from(value: Error) -> Self {
        ranked_prolly_tree::Error::Encoding(value.to_string())
    }
}
