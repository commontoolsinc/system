use blake3::HexError;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinError;
use tracing::subscriber::SetGlobalDefaultError;

/// Errors from various builder operations.
#[derive(Debug, Error)]
pub enum BuilderError {
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

impl From<std::net::AddrParseError> for BuilderError {
    fn from(value: std::net::AddrParseError) -> Self {
        BuilderError::InvalidConfiguration(format!("{}", value))
    }
}

impl From<std::io::Error> for BuilderError {
    fn from(value: std::io::Error) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<SetGlobalDefaultError> for BuilderError {
    fn from(value: SetGlobalDefaultError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<StorageError> for BuilderError {
    fn from(value: StorageError) -> Self {
        error!("{}", value);
        BuilderError::ModuleNotFound
    }
}

impl From<TransactionError> for BuilderError {
    fn from(value: TransactionError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<TableError> for BuilderError {
    fn from(value: TableError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<CommitError> for BuilderError {
    fn from(value: CommitError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<DatabaseError> for BuilderError {
    fn from(value: DatabaseError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<HexError> for BuilderError {
    fn from(_value: HexError) -> Self {
        BuilderError::BadRequest
    }
}

impl From<JoinError> for BuilderError {
    fn from(value: JoinError) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

impl From<anyhow::Error> for BuilderError {
    fn from(value: anyhow::Error) -> Self {
        error!("{}", value);
        BuilderError::Internal(format!("{}", value))
    }
}

/// Response used to contain errors in the builder server.
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    error: String,
}
