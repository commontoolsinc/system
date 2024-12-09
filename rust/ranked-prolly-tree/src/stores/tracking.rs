use crate::{BlockStore, Error, Hash, HashRef, Result};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Stats {
    reads: usize,
    writes: usize,
}

/// An LRU cache that wraps another [`BlockStore`] implementation.
#[derive(Clone)]
pub struct TrackingStore<S: BlockStore> {
    store: S,
    stats: Arc<Mutex<Stats>>,
}

impl<S> TrackingStore<S>
where
    S: BlockStore,
{
    /// Create a new [`TrackingStore`], wrapping `store`.
    pub fn new(store: S) -> Self {
        let stats = Arc::from(Mutex::from(Stats::default()));
        Self { store, stats }
    }

    /// Returns the number of reads performed on this store.
    pub fn reads(&self) -> Result<usize> {
        Ok(self
            .stats
            .lock()
            .map_err(|e| Error::Internal(e.to_string()))?
            .reads)
    }

    /// Returns the number of writes performed on this store.
    pub fn writes(&self) -> Result<usize> {
        Ok(self
            .stats
            .lock()
            .map_err(|e| Error::Internal(e.to_string()))?
            .writes)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S> BlockStore for TrackingStore<S>
where
    S: BlockStore,
{
    async fn get_block(&self, hash: &HashRef) -> Result<Option<Vec<u8>>> {
        {
            let mut stats = self
                .stats
                .lock()
                .map_err(|e| Error::Internal(e.to_string()))?;
            stats.reads += 1;
        }
        self.store.get_block(hash).await
    }

    async fn set_block(&mut self, hash: Hash, bytes: Vec<u8>) -> Result<()> {
        {
            let mut stats = self
                .stats
                .lock()
                .map_err(|e| Error::Internal(e.to_string()))?;
            stats.writes += 1;
        }
        self.store.set_block(hash.clone(), bytes.clone()).await
    }
}
