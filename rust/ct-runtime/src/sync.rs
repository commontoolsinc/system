//! Cross-target async bound compatability helpers.
//!
//! The traits herein are intended for use as a compatiblity mechanism for async
//! code that may target both `wasm32-unknown-unknown` as well as native targets
//! where the implementor may be shared across threads.
#![allow(dead_code)]
#![allow(missing_docs)]

#[cfg(not(target_arch = "wasm32"))]
pub trait ConditionalSend: Send {}

#[cfg(not(target_arch = "wasm32"))]
impl<S> ConditionalSend for S where S: Send {}

#[cfg(not(target_arch = "wasm32"))]
pub trait ConditionalSync: Send + Sync {}

#[cfg(not(target_arch = "wasm32"))]
impl<S> ConditionalSync for S where S: Send + Sync {}

#[cfg(target_arch = "wasm32")]
pub trait ConditionalSend {}

#[cfg(target_arch = "wasm32")]
impl<S> ConditionalSend for S {}

#[cfg(target_arch = "wasm32")]
pub trait ConditionalSync {}

#[cfg(target_arch = "wasm32")]
impl<S> ConditionalSync for S {}
