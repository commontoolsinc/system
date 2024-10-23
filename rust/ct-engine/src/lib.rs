//! Bindings into the Common Runtime for non-Rust-native runtimes

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod ffi;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use ffi::*;
