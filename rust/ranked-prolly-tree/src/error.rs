use crate::HashDisplay;
use std::fmt::Debug;
use thiserror::Error;

/// [`std::result::Result`] type with [Error] as its error.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Errors occurring from this crate.
#[derive(Error, PartialEq, Debug)]
pub enum Error {
    /// An operation was performed on a non-Branch node.
    #[error("Operation may only be performed on tree branches.")]
    BranchOnly,
    /// An operation attempted to use an empty list of children.
    #[error("Invalid attempt constructing a node with no children.")]
    EmptyChildren,
    /// An error occurred during encoding.
    #[error("Encoding error: {0}")]
    Encoding(String),
    /// An error occurred while reading/writing from storage or a writer.
    #[error("IO Error: {0}")]
    Io(String),
    /// A page could not be read from storage.
    #[error("Missing block: {0}")]
    MissingBlock(HashDisplay),
    /// An operation was performed on a non-Segment node.
    #[error("Operation may only be performed on tree segments.")]
    SegmentOnly,
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

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Error::Io(value.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<rexie::Error> for Error {
    fn from(value: rexie::Error) -> Self {
        Error::Io(value.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        if let Ok(js_string) = js_sys::JSON::stringify(&value) {
            Error::Io(format!("{}", js_string.as_string().unwrap()))
        } else {
            Error::Io("Non-string JsValue error.".into())
        }
    }
}
