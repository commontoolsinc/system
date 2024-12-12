use crate::{compute_rank, Adoptable, Error, Hash, HashRef, Key, Node, Rank, Result, Storage};
use async_trait::async_trait;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};

/// A key-value entry in a tree.
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry<K> {
    /// The key in this key/value pair.
    pub key: K,
    /// The value in this key/value pair.
    pub value: Vec<u8>,
}

impl<K> Entry<K>
where
    K: Key,
{
    /// Create a new [`Entry`].
    pub fn new(key: K, value: Vec<u8>) -> Self {
        Entry { key, value }
    }

    /// Computes the rank of the [`Entry`]'s key.
    pub fn rank(&self, factor: u32) -> Rank {
        compute_rank(self.key.as_ref(), factor)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, K> Adoptable<P, K> for Entry<K>
where
    K: Key,
{
    async fn adopt(children: NonEmpty<Self>, storage: &mut impl Storage) -> Result<Node<P, K>> {
        Node::segment(children, storage).await
    }
}

/// A serializable reference to a [`Node`].
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeRef<K> {
    boundary: K,
    hash: Hash,
}

impl<K> NodeRef<K>
where
    K: Key,
{
    pub(crate) fn new(hash: Hash, boundary: K) -> Self {
        NodeRef { hash, boundary }
    }

    pub(crate) fn hash(&self) -> &HashRef {
        &self.hash
    }

    pub(crate) fn boundary(&self) -> &K {
        &self.boundary
    }

    /// Computes the rank of this [`NodeRef`].
    pub fn rank(&self, factor: u32) -> Rank {
        compute_rank(self.boundary().as_ref(), factor)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, K> Adoptable<P, K> for NodeRef<K>
where
    K: Key,
{
    async fn adopt(children: NonEmpty<Self>, storage: &mut impl Storage) -> Result<Node<P, K>> {
        Node::branch(children, storage).await
    }
}

/// The serializable construct representing a [`Node`].
/// A [`Block`] is what is stored in a [`BlockStore`],
/// used to hydrate and store nodes.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum Block<K> {
    Branch(NonEmpty<NodeRef<K>>),
    Segment(NonEmpty<Entry<K>>),
}

impl<K> Block<K>
where
    K: Key,
{
    pub fn branch(data: NonEmpty<NodeRef<K>>) -> Self {
        Block::Branch(data)
    }

    pub fn segment(data: NonEmpty<Entry<K>>) -> Self {
        Block::Segment(data)
    }

    pub fn boundary(&self) -> &K {
        match self {
            Block::Branch(data) => &data.last().boundary,
            Block::Segment(data) => &data.last().key,
        }
    }

    pub fn node_refs(&self) -> Result<&NonEmpty<NodeRef<K>>> {
        match self {
            Block::Branch(data) => Ok(&data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    pub fn into_node_refs(self) -> Result<NonEmpty<NodeRef<K>>> {
        match self {
            Block::Branch(data) => Ok(data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    pub fn entries(&self) -> Result<&NonEmpty<Entry<K>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(&data),
        }
    }

    pub fn into_entries(self) -> Result<NonEmpty<Entry<K>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(data),
        }
    }

    pub async fn encode(&self, storage: &mut impl Storage) -> Result<NodeRef<K>> {
        let hash = storage.write(&self).await?;
        let boundary = self.boundary();
        Ok(NodeRef::new(hash, boundary.to_owned()))
    }

    pub async fn decode(hash: &HashRef, storage: &impl Storage) -> Result<Option<Block<K>>> {
        storage.read(hash).await
    }
}
