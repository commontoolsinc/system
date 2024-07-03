#![warn(missing_docs)]

//! A library containing shared tracing functionality across common crates.

#[allow(unused_imports)]
#[macro_use]
extern crate common_macros;

/// Contains implementation for the `#[common_tracing]` macro.
/// Prefer using the [common_tracing::common_tracing] macro over
/// calling these functions directly.
pub mod macro_impl;

pub use common_macros::common_tracing;
