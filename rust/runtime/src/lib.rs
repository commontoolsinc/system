#![warn(missing_docs)]

//! A library that constitutes the substantial implementation of a Common Runtime.

// TODO: All [async_trait::async_trait]-using code must be made compatible with
// the `wasm32-unknown-unknown` target.

#[macro_use]
extern crate tracing;

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

mod io;
pub use io::*;

mod wit;
pub use wit::*;

pub mod sync;
