use std::fmt::Debug;
use thiserror::Error;

/// [`std::result::Result`] type with [Error]
/// as its error.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Various errors that may be encountered when invoking runtime code.
#[derive(Error, PartialEq, Debug)]
pub enum Error {
    /// An error occurred in the Runtime.
    #[error("Runtime error: {0}")]
    RuntimeError(ct_runtime::Error),

    /// Could not find requested runtime module.
    #[error("Module not found with id {0}")]
    ModuleNotFound(ct_common::ModuleId),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<ct_runtime::Error> for Error {
    fn from(value: ct_runtime::Error) -> Self {
        Error::RuntimeError(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::InternalError(value)
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Error> for wasm_bindgen::JsValue {
    fn from(error: Error) -> Self {
        error.to_string().into()
    }
}
