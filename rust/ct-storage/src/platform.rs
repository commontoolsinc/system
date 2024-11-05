use ranked_prolly_tree::{BincodeEncoder, NodeStorage, Result};

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use super::*;
    use ranked_prolly_tree::FileSystemStore;
    use std::path::PathBuf;

    pub type PlatformStorage = NodeStorage<BincodeEncoder, FileSystemStore>;
    pub type PlatformStorageParams = PathBuf;

    pub async fn open_platform_storage(params: PlatformStorageParams) -> Result<PlatformStorage> {
        let store = FileSystemStore::new(params).await?;
        Ok(NodeStorage::new(BincodeEncoder::default(), store))
    }
}

#[cfg(target_arch = "wasm32")]
mod inner {
    use super::*;
    use ranked_prolly_tree::{IndexedDbStore, LruStore};

    pub type PlatformStorage = NodeStorage<BincodeEncoder, LruStore<IndexedDbStore>>;
    pub type PlatformStorageParams = (String, String);

    pub async fn open_platform_storage(params: PlatformStorageParams) -> Result<PlatformStorage> {
        let idb = IndexedDbStore::new(&params.0, &params.1).await?;
        let cache = LruStore::new(idb, 10_000)?;
        Ok(NodeStorage::new(BincodeEncoder::default(), cache))
    }
}

pub use inner::*;
