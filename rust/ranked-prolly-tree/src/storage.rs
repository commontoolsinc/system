use crate::{BasicEncoder, Block, BlockStore, Encoder, Hash, HashRef, Key, MemoryStore, Result};
use async_trait::async_trait;
use ct_common::ConditionalSync;
use std::marker::PhantomData;

/// Trait representing the encoding and storage of data
/// for nodes.
///
/// A blanket implementation is provided for implementors
/// of both [`Encoder`] and [`BlockStore`].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Storage<K, V>
where
    Self: Encoder<K, V> + BlockStore,
    K: Key,
    V: ConditionalSync,
{
    /// Encodes `item` to storage.
    async fn write(&mut self, block: &Block<K, V>) -> Result<Hash>
    where
        K: 'static,
    {
        let (hash, bytes) = self.encode(block)?;
        self.set_block(hash.clone(), bytes).await?;
        Ok(hash)
    }

    /// Decodes item from storage.
    async fn read(&self, hash: &HashRef) -> Result<Option<Block<K, V>>> {
        let Some(bytes) = self.get_block(hash).await? else {
            return Ok(None);
        };
        Ok(Some(self.decode(&bytes)?))
    }
}

impl<T, K, V> Storage<K, V> for T
where
    K: Key + 'static,
    V: ConditionalSync,
    T: Encoder<K, V> + BlockStore,
{
}

/// A [`Storage`] implementation comprised of an underlying [`Encoder`]
/// and [`BlockStore`].
#[derive(Clone)]
pub struct NodeStorage<K, V, E, B> {
    encoder: E,
    store: B,
    marker_k: PhantomData<K>,
    marker_v: PhantomData<V>,
}

impl<K, V, E, B> NodeStorage<K, V, E, B> {
    /// Creates a new [`NodeStorage`] from an [`Encoder`] and [`BlockStore`].
    pub fn new(encoder: E, store: B) -> Self {
        Self {
            encoder,
            store,
            marker_k: PhantomData,
            marker_v: PhantomData,
        }
    }
}

impl<K, V, E, B> Default for NodeStorage<K, V, E, B>
where
    K: Key,
    E: Encoder<K, V> + Default,
    B: BlockStore + Default,
{
    fn default() -> NodeStorage<K, V, E, B> {
        NodeStorage::new(E::default(), B::default())
    }
}

impl<K, V, E, B> Encoder<K, V> for NodeStorage<K, V, E, B>
where
    K: Key,
    E: Encoder<K, V> + Default,
    B: BlockStore,
    V: Clone + ConditionalSync,
{
    fn encode(&self, block: &Block<K, V>) -> Result<(Hash, Vec<u8>)> {
        self.encoder.encode(block)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Block<K, V>> {
        self.encoder.decode(bytes)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<K, V, E, B> BlockStore for NodeStorage<K, V, E, B>
where
    K: Key,
    V: Clone + ConditionalSync,
    E: Encoder<K, V>,
    B: BlockStore,
{
    async fn get_block(&self, key: &HashRef) -> Result<Option<Vec<u8>>> {
        self.store.get_block(key).await
    }

    async fn set_block(&mut self, key: Hash, bytes: Vec<u8>) -> Result<()> {
        self.store.set_block(key, bytes).await
    }
}

/// An alias type for [`NodeStorage`] with [`BasicEncoder`] and [`MemoryStore`].
pub type EphemeralStorage<K, V> = NodeStorage<K, V, BasicEncoder, MemoryStore>;
