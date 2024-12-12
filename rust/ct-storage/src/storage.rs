use crate::{encoding::ColumnarEncoder, Key, PlatformStorage, Result};
use async_stream::try_stream;
use futures_core::Stream;
use ranked_prolly_tree::{Entry, MemoryStore, NodeStorage, Storage, Tree};
use std::ops::RangeBounds;

#[cfg(not(target_arch = "wasm32"))]
use crate::platform::open_fs_storage;
#[cfg(target_arch = "wasm32")]
use crate::platform::open_idb_storage;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

const BRANCHING_FACTOR: u8 = 64;

/// Passive database.
pub struct CtStorage<S> {
    tree: Tree<BRANCHING_FACTOR, S, Key>,
}

impl<S> CtStorage<S>
where
    S: Storage<Key, Vec<u8>>,
{
    /// Opens a file system backed database, optionally from a root hash.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn open_fs(
        path: PathBuf,
        root: Option<Vec<u8>>,
    ) -> Result<CtStorage<PlatformStorage>> {
        let storage = open_fs_storage(path).await?;
        let tree = create_tree(storage, root).await?;
        Ok(CtStorage { tree })
    }

    /// Opens an IndexedDb backed database, optionally from a root hash.
    #[cfg(target_arch = "wasm32")]
    pub async fn open_idb(
        db_name: String,
        store_name: String,
        root: Option<Vec<u8>>,
    ) -> Result<CtStorage<PlatformStorage>> {
        let storage = open_idb_storage(db_name, store_name).await?;
        let tree = create_tree(storage, root).await?;
        Ok(CtStorage { tree })
    }

    /// Opens a new storage instance backed by a memory store.
    pub fn open_memory(
    ) -> Result<CtStorage<NodeStorage<Key, Vec<u8>, ColumnarEncoder, MemoryStore>>> {
        let storage = NodeStorage::new(ColumnarEncoder::default(), MemoryStore::default());
        let tree = Tree::new(storage);
        Ok(CtStorage { tree })
    }

    /// Retrieves the current root hash.
    pub fn hash(&self) -> Option<&[u8]> {
        self.tree.hash()
    }

    /// Retrieves the value associated with `key` from the tree.
    pub async fn get(&self, key: &Key) -> Result<Option<Vec<u8>>> {
        Ok(self.tree.get(key).await?)
    }

    /// Sets a `key`/`value` pair into the tree.
    pub async fn set(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        Ok(self.tree.set(key, value).await?)
    }

    /// Returns an async stream over entries with keys within the provided range.
    pub async fn get_range<'a, R>(
        &'a self,
        range: R,
    ) -> impl Stream<Item = Result<Entry<Key, Vec<u8>>>> + 'a
    where
        R: RangeBounds<&'a Key> + 'a,
    {
        try_stream! {
            let stream = self.tree.get_range(range).await;
            for await item in stream {
                yield item?;
            }
        }
    }
}

async fn create_tree<S: Storage<Key, Vec<u8>>>(
    storage: S,
    root: Option<Vec<u8>>,
) -> Result<Tree<BRANCHING_FACTOR, S, Key, Vec<u8>>> {
    Ok(match root {
        Some(hash) => Tree::from_hash(&hash, storage).await?,
        None => Tree::new(storage),
    })
}
