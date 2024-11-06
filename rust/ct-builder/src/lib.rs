#![cfg(not(target_arch = "wasm32"))]
#![warn(missing_docs)]
//! Utilities for compiling/bundling JavaScript into
//! a single source.

#[macro_use]
extern crate tracing;

mod artifact;
mod builder;
mod bundle;
mod error;
mod import_map;
mod serve;
mod storage;

pub use builder::*;
pub use bundle::*;
pub use error::*;
pub use import_map::*;
pub use serve::*;
