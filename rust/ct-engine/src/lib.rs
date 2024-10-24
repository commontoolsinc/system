//! Root component representing the Common Engine.

mod engine;
mod error;
mod sandbox;

pub use engine::*;
pub use error::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod ffi;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use ffi::*;
