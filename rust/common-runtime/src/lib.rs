#![warn(missing_docs)]

//! A library that constitutes the substantial implementation of a Common Runtime.

// TODO: All [async_trait::async_trait]-using code must be made compatible with
// the `wasm32-unknown-unknown` target.

#[macro_use]
extern crate tracing;

mod components;
pub use components::*;

pub mod sandbox;

pub mod runtime_experiment;

mod error;
pub use error::*;

mod content_type;
pub use content_type::*;

mod schedule;
pub use schedule::*;

mod value;
pub use value::*;

mod module;
pub use module::*;

#[cfg(not(target_arch = "wasm32"))]
mod serve;
#[cfg(not(target_arch = "wasm32"))]
pub use serve::*;

mod io;
pub use io::*;

pub mod runtime;

pub mod sync;
