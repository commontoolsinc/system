#![warn(missing_docs)]

//! A library that constitutes the substantial implementation of a Common Runtime.

// TODO: All [`async_trait::async_trait`]-using code must be made compatible with
// the `wasm32-unknown-unknown` target.

#[macro_use]
extern crate tracing;

mod data;
pub use data::*;

mod error;
pub use error::*;

mod content_type;
pub use content_type::*;

#[cfg(not(target_arch = "wasm32"))]
mod serve;
#[cfg(not(target_arch = "wasm32"))]
pub use serve::*;

#[cfg(target_arch = "wasm32")]
pub mod ffi;

pub mod sync;

mod policy;
pub use policy::*;

mod value;
pub use value::*;

mod cache;
pub use cache::*;

mod module;
pub use module::*;

mod runtime;
pub use runtime::*;

#[cfg(all(not(target_arch = "wasm32"), any(test, feature = "helpers")))]
pub mod helpers;
