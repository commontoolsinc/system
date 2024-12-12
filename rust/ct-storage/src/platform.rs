use crate::Key;
use ranked_prolly_tree::{BincodeEncoder, NodeStorage, Result};

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use super::*;
    use ranked_prolly_tree::FileSystemStore;
    use std::path::PathBuf;

    /// Default persistent platform storage for the current platform.
    pub type PlatformStorage = NodeStorage<Key, BincodeEncoder, FileSystemStore>;

    pub async fn open_fs_storage(params: PathBuf) -> Result<PlatformStorage> {
        let store = FileSystemStore::new(params).await?;
        Ok(NodeStorage::new(BincodeEncoder::default(), store))
    }
}

#[cfg(target_arch = "wasm32")]
mod inner {
    use super::*;
    use ranked_prolly_tree::{IndexedDbStore, LruStore};

    /// Default persistent platform storage for the current platform.
    pub type PlatformStorage = NodeStorage<Key, BincodeEncoder, LruStore<IndexedDbStore>>;

    pub async fn open_idb_storage(db_name: String, store_name: String) -> Result<PlatformStorage> {
        let idb = IndexedDbStore::new(&db_name, &store_name).await?;
        let cache = LruStore::new(idb, 10_000)?;
        Ok(NodeStorage::new(BincodeEncoder::default(), cache))
    }
}

pub use inner::*;
