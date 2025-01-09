//! Root component representing the Common Engine.

#[cfg(feature = "runtime")]
mod engine;
mod error;
#[cfg(feature = "runtime")]
mod sandbox;

#[cfg(feature = "runtime")]
pub use engine::*;
pub use error::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod ffi;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use ffi::*;
