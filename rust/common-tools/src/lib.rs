#![warn(missing_docs)]

//! CLI tools for the Common Runtime.

#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod commands;
#[cfg(not(target_arch = "wasm32"))]
pub use cli::*;
#[cfg(not(target_arch = "wasm32"))]
pub use commands::*;
