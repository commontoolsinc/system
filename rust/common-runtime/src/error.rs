use crate::ModuleInstanceId;
use common_ifc::CommonIfcError;
use std::fmt::Debug;
use thiserror::Error;

/// [`std::result::Result`] type with [CommonRuntimeError]
/// as its error.
pub type Result<T> = ::std::result::Result<T, CommonRuntimeError>;

/// Various errors that may be encountered when invoking runtime code.
#[derive(Error, PartialEq, Debug)]
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

    /// A specified Common Module ID was not valid
    #[error("Invalid Common Module ID: {0}")]
    InvalidModuleId(String),

    /// The specified Common Module instance ID did not correspond to a living
    /// instance
    #[error("Unknown Common Module instance ID: {0}")]
    UnknownInstanceId(ModuleInstanceId),

    /// A provided Value was empty or of an unexpected shape
    #[error("Invalid Value")]
    InvalidValue,

    /// A provided ValueKind was not recognized
    #[error("Invalid or unsupported kind of value: {0}")]
    InvalidValueKind(String),

    /// The provided module sources were missing or otherwise invalid
    #[error("Invalid module source: {0}")]
    InvalidModuleSource(String),

    /// The provided instantiation parameters are not supported
    #[error("Invalid instantiation parameters: {0}")]
    InvalidInstantiationParameters(String),

    /// There was a policy failure.
    #[error("Policy rejected invocation: {0}")]
    PolicyRejection(CommonIfcError),
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tonic::transport::Error> for CommonRuntimeError {
    fn from(value: tonic::transport::Error) -> Self {
        CommonRuntimeError::InternalError(format!("{value}"))
    }
}

impl From<CommonIfcError> for CommonRuntimeError {
    fn from(value: CommonIfcError) -> Self {
        CommonRuntimeError::PolicyRejection(value)
    }
}
