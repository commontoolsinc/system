use blake3::HexError;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use tokio::task::JoinError;
use tracing::subscriber::SetGlobalDefaultError;

/// Errors from various builder operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from a bad request body.
    #[error("Bad request body")]
    BadRequest,
    /// Error from an invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    /// Error from a module being invalid.
    #[error("Invalid module: {0}")]
    InvalidModule(String),
    /// Error from a module that was not found.
    #[error("Module not found")]
    ModuleNotFound,
    /// Catch-all generic error for builder services.
    #[error("An internal error occurred")]
    Internal(String),
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Error::InvalidConfiguration(format!("{}", value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<SetGlobalDefaultError> for Error {
    fn from(value: SetGlobalDefaultError) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<StorageError> for Error {
    fn from(value: StorageError) -> Self {
        error!("{}", value);
        Error::ModuleNotFound
    }
}

impl From<TransactionError> for Error {
    fn from(value: TransactionError) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<TableError> for Error {
    fn from(value: TableError) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<CommitError> for Error {
    fn from(value: CommitError) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<DatabaseError> for Error {
    fn from(value: DatabaseError) -> Self {
        error!("Database error: {}", value);
        Error::Internal(format!("Database error: {}", value))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Error::Internal(value.to_string())
    }
}

impl From<HexError> for Error {
    fn from(_value: HexError) -> Self {
        Error::BadRequest
    }
}

impl From<JoinError> for Error {
    fn from(value: JoinError) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        error!("{}", value);
        Error::Internal(format!("{}", value))
    }
}
