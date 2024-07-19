#![cfg(not(target_arch = "wasm32"))]
#![warn(missing_docs)]
//! Utilities for compiling/bundling JavaScript into
//! a single source.

#[macro_use]
extern crate tracing;

mod bake;
mod bundle;
mod error;
mod serve;
mod storage;

pub use bake::*;
pub use bundle::*;
pub use error::*;
pub use serve::serve;
