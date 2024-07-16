//! Utilities for compiling/bundling JavaScript into
//! a single source.

mod bundler;
mod polyfill;
mod wasi_shim;

pub use bundler::JavaScriptBundler;
pub use polyfill::polyfill;
