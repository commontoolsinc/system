//! This module contains implementations of sandboxing based on the Core Wasm
//! APIs that broadly available in web browsers.

mod compile;
pub use compile::*;

mod interpret;
pub use interpret::*;
