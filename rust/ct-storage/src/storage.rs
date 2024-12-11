use crate::platform::{open_platform_storage, PlatformStorage, PlatformStorageParams};
use futures_core::Stream;
use ranked_prolly_tree::{Entry, Result, Tree};
use std::ops::RangeBounds;

const BRANCHING_FACTOR: u8 = 64;

/// Passive database.
pub struct Storage {
    tree: Tree<BRANCHING_FACTOR, PlatformStorage>,
}

impl Storage {
    /// Opens a database, using the provided root hash if provided.
    pub async fn open(params: PlatformStorageParams, root: Option<Vec<u8>>) -> Result<Self> {
        let inner = open_platform_storage(params).await?;

        let tree = match root {
            Some(hash) => Tree::from_hash(&hash, inner).await?,
            None => Tree::new(inner),
        };
        Ok(Storage { tree })
    }

    /// Retrieves the current root hash.
    pub fn hash(&self) -> Option<&[u8]> {
        self.tree.hash()
    }

    /// Retrieves the value associated with `key` from the tree.
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.tree.get(key).await
    }

    /// Sets a `key`/`value` pair into the tree.
    pub async fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.tree.set(key, value).await
    }

    /// Returns an async stream over all entries.
    pub async fn stream<'a>(&'a self) -> impl Stream<Item = Result<Entry>> + 'a {
        self.tree.stream().await
    }

    /// Returns an async stream over entries with keys within the provided range.
    pub async fn get_range<'a, R>(&'a self, range: R) -> impl Stream<Item = Result<Entry>> + 'a
    where
        R: RangeBounds<&'a [u8]> + 'a,
    {
        self.tree.get_range(range).await
    }
}
