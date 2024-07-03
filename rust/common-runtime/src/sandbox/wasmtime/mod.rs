//! This module contains implementations of sandboxing based on the Wasmtime
//! project. Wasmtime sandboxes are expected to run outside of web browsers,
//! either on a user's local device in a separate process or else on a
//! network-connected remote computer that is considered to be within the user's
//! logical trust domain.

pub mod bindings;

mod compile;
pub use compile::*;

mod interpret;
// pub use interpreter::*;
