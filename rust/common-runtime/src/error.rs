use std::fmt::Debug;
use thiserror::Error;

/// Various errors that may be encountered when invoking runtime code.
#[derive(Error, Debug)]
pub enum CommonRuntimeError {
    /// A Wasm Component failed to prepare
    #[error("Failed to prepare a Wasm Component: {0}")]
    PreparationFailed(String),

    /// A Wasm Component failed to link
    #[error("Failed to link a Wasm Component: {0}")]
    LinkFailed(String),

    /// A sandbox failed to be created
    #[error("Failed to instantiate a sandbox: {0}")]
    SandboxCreationFailed(String),

    /// A Common Module failed to be instantiated
    #[error("Failed to instantiate a Common Module: {0}")]
    ModuleInstantiationFailed(String),

    /// An error occurred when a Common Module was run
    #[error("Failed to run a Common Module: {0}")]
    ModuleRunFailed(String),

    /// An unexpected internal error occurred
    #[error("Internal error")]
    InternalError(String),

    /// A provided value was empty or of an unexpected shape
    #[error("Invalid value")]
    InvalidValue,
}

impl From<tonic::transport::Error> for CommonRuntimeError {
    fn from(value: tonic::transport::Error) -> Self {
        CommonRuntimeError::InternalError(format!("{value}"))
    }
}
