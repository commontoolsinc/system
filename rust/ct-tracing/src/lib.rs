#![warn(missing_docs)]

//! A library containing shared tracing functionality across ct crates.

#[allow(unused_imports)]
#[macro_use]
extern crate ct_macros;

/// Contains implementation for the `#[ct_tracing]` macro.
/// Prefer using the [`ct_tracing`] macro over
/// calling these functions directly.
pub mod macro_impl;

pub use ct_macros::ct_tracing;
