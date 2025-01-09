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
    #[cfg(feature = "runtime")]
    RuntimeError(ct_runtime::Error),

    /// Could not find requested runtime module.
    #[error("Module not found with id {0}")]
    ModuleNotFound(ct_common::ModuleId),

    /// Error in the storage layer.
    #[error("Storage error: {0}")]
    #[cfg(feature = "storage")]
    StorageError(ct_storage::Error),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(feature = "runtime")]
impl From<ct_runtime::Error> for Error {
    fn from(value: ct_runtime::Error) -> Self {
        Error::RuntimeError(value)
    }
}

#[cfg(feature = "storage")]
impl From<ct_storage::Error> for Error {
    fn from(value: ct_storage::Error) -> Self {
        Error::StorageError(value)
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

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(error: wasm_bindgen::JsValue) -> Self {
        const JS_ERROR: &str = "UNKNOWN JS ERROR";
        match error.as_string() {
            Some(string) => string.into(),
            None => String::from(JS_ERROR).into(),
        }
    }
}
