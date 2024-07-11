#![warn(missing_docs)]

//! A library that constitutes the substantial implementation of a Common Runtime.

// TODO: All [async_trait::async_trait]-using code must be made compatible with
// the `wasm32-unknown-unknown` target.

#[macro_use]
extern crate tracing;

mod components;
pub use components::*;

mod sandbox;
pub use sandbox::*;

mod error;
pub use error::*;

mod content_type;
pub use content_type::*;

mod value;
pub use value::*;

mod module;
pub use module::*;

mod serve;
pub use serve::*;

mod io;
pub use io::*;

mod runtime;
pub use runtime::*;

pub mod sync;
