use crate::{BlockStore, Error, Hash, HashRef, Result};
use async_trait::async_trait;
use lru::LruCache;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;

/// An LRU cache that wraps another [`BlockStore`] implementation.
#[derive(Clone)]
pub struct LruStore<S> {
    store: S,
    cache: Arc<Mutex<LruCache<Hash, Vec<u8>>>>,
}

impl<S> LruStore<S>
where
    S: BlockStore,
{
    /// Create a new [`LruStore`], wrapping `store`, with `cache_size`.
    ///
    /// `cache_size` must be a non-zero value.
    pub fn new(store: S, cache_size: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(cache_size)
            .ok_or_else(|| Error::Internal("LruStore requires non-zero cache size.".into()))?;
        let cache = Arc::from(Mutex::from(LruCache::new(cache_size)));
        Ok(Self { store, cache })
    }

    async fn from_cache(&self, hash: &HashRef) -> Option<Vec<u8>> {
        let mut cache = self.cache.lock().await;
        cache.get(hash).map(|cached| cached.to_owned())
    }

    async fn set_cache(&self, hash: Hash, value: Vec<u8>) {
        let mut cache = self.cache.lock().await;
        cache.put(hash, value);
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S> BlockStore for LruStore<S>
where
    S: BlockStore,
{
    async fn get_block(&self, hash: &HashRef) -> Result<Option<Vec<u8>>> {
        if let Some(cached) = self.from_cache(hash).await {
            return Ok(Some(cached));
        }
        let Some(block) = self.store.get_block(hash).await? else {
            return Ok(None);
        };
        self.set_cache(hash.to_owned(), block.clone()).await;
        Ok(Some(block))
    }

    async fn set_block(&mut self, hash: Hash, bytes: Vec<u8>) -> Result<()> {
        self.store.set_block(hash.clone(), bytes.clone()).await?;
        self.set_cache(hash, bytes).await;
        Ok(())
    }
}
