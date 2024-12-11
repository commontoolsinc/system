use crate::{BincodeEncoder, BlockStore, Encoder, Hash, HashRef, MemoryStore, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Trait representing the encoding and storage of data
/// for nodes.
///
/// A blanket implementation is provided for implementors
/// of both [`Encoder`] and [`BlockStore`].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Storage: Encoder + BlockStore {
    /// Encodes `item` to storage.
    async fn write(&mut self, item: impl Serialize + Send) -> Result<Hash> {
        let (hash, bytes) = self.encode(item)?;
        self.set_block(hash.clone(), bytes).await?;
        Ok(hash)
    }

    /// Decodes item from storage.
    async fn read<T: DeserializeOwned>(&self, hash: &HashRef) -> Result<Option<T>> {
        let Some(bytes) = self.get_block(hash).await? else {
            return Ok(None);
        };
        Ok(Some(self.decode(&bytes)?))
    }
}

impl<T> Storage for T where T: Encoder + BlockStore {}

/// A [`Storage`] implementation comprised of an underlying [`Encoder`]
/// and [`BlockStore`].
#[derive(Clone)]
pub struct NodeStorage<E: Encoder, B: BlockStore> {
    encoder: E,
    store: B,
}

impl<E, B> NodeStorage<E, B>
where
    E: Encoder,
    B: BlockStore,
{
    /// Creates a new [`NodeStorage`] from an [`Encoder`] and [`BlockStore`].
    pub fn new(encoder: E, store: B) -> Self {
        Self { encoder, store }
    }
}

impl<E, B> Default for NodeStorage<E, B>
where
    E: Encoder + Default,
    B: BlockStore + Default,
{
    fn default() -> NodeStorage<E, B> {
        NodeStorage::new(E::default(), B::default())
    }
}

impl<E, B> Encoder for NodeStorage<E, B>
where
    E: Encoder,
    B: BlockStore,
{
    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        self.encoder.decode(bytes)
    }
    fn encode(&self, item: impl Serialize) -> Result<(Hash, Vec<u8>)> {
        self.encoder.encode(item)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<E, B> BlockStore for NodeStorage<E, B>
where
    E: Encoder,
    B: BlockStore,
{
    async fn get_block(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.store.get_block(key).await
    }

    async fn set_block(&mut self, key: Vec<u8>, bytes: Vec<u8>) -> Result<()> {
        self.store.set_block(key, bytes).await
    }
}

/// An alias type for [`NodeStorage`] with [`BincodeEncoder`] and [`MemoryStore`].
#[cfg(feature = "bincode")]
pub type EphemeralStorage = NodeStorage<BincodeEncoder, MemoryStore>;
