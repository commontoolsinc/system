use crate::{Block, Key, Result};
use async_trait::async_trait;
use ct_common::ConditionalSync;

#[cfg(feature = "basic-encoder")]
mod basic;
mod hash;
#[cfg(feature = "basic-encoder")]
pub mod io;

#[cfg(feature = "basic-encoder")]
pub use basic::*;
pub use hash::*;

/// Trait responsible for encoding data into/from bytes, and producing
/// a [`Hash`] that can be used to uniquely reference it.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Encoder<K: Key, V>: Clone + ConditionalSync {
    /// Encode a serializable item into its referencable [`Hash`] and its bytes.
    fn encode(&self, block: &Block<K, V>) -> Result<(Hash, Vec<u8>)>;

    /// Decode bytes into a `Block`.
    fn decode(&self, bytes: &[u8]) -> Result<Block<K, V>>;
}
