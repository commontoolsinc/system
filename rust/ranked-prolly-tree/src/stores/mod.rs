use crate::{Hash, HashRef, Result};
use async_trait::async_trait;
use ct_common::ConditionalSync;

#[cfg(not(target_arch = "wasm32"))]
mod fs;
#[cfg(target_arch = "wasm32")]
mod indexed_db;
#[cfg(feature = "lru")]
mod lru;
mod memory;
mod tracking;

#[cfg(not(target_arch = "wasm32"))]
pub use fs::*;
#[cfg(target_arch = "wasm32")]
pub use indexed_db::*;
#[cfg(feature = "lru")]
pub use lru::*;
pub use memory::*;
pub use tracking::*;

/// Abstraction for storing blocks of data by hash.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait BlockStore: Clone + ConditionalSync {
    /// Retrieve a block by its hash.
    async fn get_block(&self, hash: &HashRef) -> Result<Option<Vec<u8>>>;
    /// Store `bytes`, keyed by its hash.
    async fn set_block(&mut self, hash: Hash, bytes: Vec<u8>) -> Result<()>;
}
