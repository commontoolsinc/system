use crate::{encoding::ColumnarEncoder, Key};
use ranked_prolly_tree::{NodeStorage, Result};

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use super::*;
    use ranked_prolly_tree::FileSystemStore;
    use std::path::PathBuf;

    /// Default persistent platform storage for the current platform.
    pub type PlatformStorage = NodeStorage<Key, Vec<u8>, ColumnarEncoder, FileSystemStore>;

    pub async fn open_fs_storage(params: PathBuf) -> Result<PlatformStorage> {
        let store = FileSystemStore::new(params).await?;
        Ok(NodeStorage::new(ColumnarEncoder::default(), store))
    }
}

#[cfg(target_arch = "wasm32")]
mod inner {
    use super::*;
    use ranked_prolly_tree::{IndexedDbStore, LruStore};

    /// Default persistent platform storage for the current platform.
    pub type PlatformStorage = NodeStorage<Key, Vec<u8>, ColumnarEncoder, LruStore<IndexedDbStore>>;

    pub async fn open_idb_storage(db_name: String, store_name: String) -> Result<PlatformStorage> {
        let idb = IndexedDbStore::new(&db_name, &store_name).await?;
        let cache = LruStore::new(idb, 10_000)?;
        Ok(NodeStorage::new(ColumnarEncoder::default(), cache))
    }
}

pub use inner::*;
