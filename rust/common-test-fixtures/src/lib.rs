#![warn(missing_docs)]

//! This crate contains test fixtures and shared helper code that is used across
//! the common-* constellation of crates

#[cfg(not(target_arch = "wasm32"))]
mod server;
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;

pub mod sources;
