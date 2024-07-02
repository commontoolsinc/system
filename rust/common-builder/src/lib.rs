#![warn(missing_docs)]
//! Utilities for compiling/bundling JavaScript into
//! a single source.

#[macro_use]
extern crate tracing;

mod bake;
mod error;
mod openapi;
mod polyfill;
mod routes;
mod serve;
mod storage;

pub use bake::*;
pub use error::*;
pub use serve::serve;
