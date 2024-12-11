use crate::{BlockStore, Error, Result};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// An in-memory [`BlockStore`] implementation.
#[derive(Default, Clone)]
pub struct SyncMemoryStore(Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>);

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl BlockStore for SyncMemoryStore {
    async fn get_block(&self, hash: &[u8]) -> Result<Option<Vec<u8>>> {
        let map = self.0.lock().map_err(|e| Error::Internal(e.to_string()))?;
        Ok(map.get(hash).map(|value| value.to_owned()))
    }

    async fn set_block(&mut self, hash: Vec<u8>, bytes: Vec<u8>) -> Result<()> {
        let mut map = self.0.lock().map_err(|e| Error::Internal(e.to_string()))?;
        map.insert(hash, bytes);
        Ok(())
    }
}

/// An in-memory [`BlockStore`] implementation.
#[derive(Default, Clone)]
pub struct MemoryStore(HashMap<Vec<u8>, Vec<u8>>);

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl BlockStore for MemoryStore {
    async fn get_block(&self, hash: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.0.get(hash).map(|value| value.to_owned()))
    }

    async fn set_block(&mut self, hash: Vec<u8>, bytes: Vec<u8>) -> Result<()> {
        self.0.insert(hash, bytes);
        Ok(())
    }
}
