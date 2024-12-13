use crate::{BincodeEncoder, Block, BlockStore, Encoder, Hash, HashRef, Key, MemoryStore, Result};
use async_trait::async_trait;
use std::marker::PhantomData;

/// Trait representing the encoding and storage of data
/// for nodes.
///
/// A blanket implementation is provided for implementors
/// of both [`Encoder`] and [`BlockStore`].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Storage<K: Key>: Encoder<K> + BlockStore {
    /// Encodes `item` to storage.
    async fn write(&mut self, block: &Block<K>) -> Result<Hash>
    where
        K: 'static,
    {
        let (hash, bytes) = self.encode(block)?;
        self.set_block(hash.clone(), bytes).await?;
        Ok(hash)
    }

    /// Decodes item from storage.
    async fn read(&self, hash: &HashRef) -> Result<Option<Block<K>>> {
        let Some(bytes) = self.get_block(hash).await? else {
            return Ok(None);
        };
        Ok(Some(self.decode(&bytes)?))
    }
}

impl<T, K> Storage<K> for T
where
    K: Key + 'static,
    T: Encoder<K> + BlockStore,
{
}

/// A [`Storage`] implementation comprised of an underlying [`Encoder`]
/// and [`BlockStore`].
#[derive(Clone)]
pub struct NodeStorage<K: Key, E: Encoder<K>, B: BlockStore> {
    encoder: E,
    store: B,
    marker: PhantomData<K>,
}

impl<K, E, B> NodeStorage<K, E, B>
where
    K: Key,
    E: Encoder<K>,
    B: BlockStore,
{
    /// Creates a new [`NodeStorage`] from an [`Encoder`] and [`BlockStore`].
    pub fn new(encoder: E, store: B) -> Self {
        Self {
            encoder,
            store,
            marker: PhantomData,
        }
    }
}

impl<K, E, B> Default for NodeStorage<K, E, B>
where
    K: Key,
    E: Encoder<K> + Default,
    B: BlockStore + Default,
{
    fn default() -> NodeStorage<K, E, B> {
        NodeStorage::new(E::default(), B::default())
    }
}

impl<K, E, B> Encoder<K> for NodeStorage<K, E, B>
where
    K: Key,
    E: Encoder<K>,
    B: BlockStore,
{
    fn encode(&self, block: &Block<K>) -> Result<(Hash, Vec<u8>)> {
        self.encoder.encode(block)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Block<K>> {
        self.encoder.decode(bytes)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<K, E, B> BlockStore for NodeStorage<K, E, B>
where
    K: Key,
    E: Encoder<K>,
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
pub type EphemeralStorage<K> = NodeStorage<K, BincodeEncoder, MemoryStore>;
