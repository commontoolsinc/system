use crate::{
    compute_rank, Adoptable, Error, Hash, HashRef, Key, KeyRef, Node, Rank, Result, Storage,
};
use async_trait::async_trait;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};

/// A key-value entry in a tree.
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The key in this key/value pair.
    pub key: Key,
    /// The value in this key/value pair.
    pub value: Vec<u8>,
}

impl Entry {
    /// Create a new [`Entry`].
    pub fn new(key: Key, value: Vec<u8>) -> Self {
        Entry { key, value }
    }

    /// Computes the rank of the [`Entry`]'s key.
    pub fn rank(&self, factor: u32) -> Rank {
        compute_rank(&self.key, factor)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Adoptable for Entry {
    async fn adopt<const P: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P>> {
        Node::segment(children, storage).await
    }
}

/// A serializable reference to a [`Node`].
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeRef {
    boundary: Key,
    hash: Hash,
}

impl NodeRef {
    pub(crate) fn new(hash: Hash, boundary: Key) -> Self {
        NodeRef { hash, boundary }
    }

    pub(crate) fn hash(&self) -> &HashRef {
        &self.hash
    }

    pub(crate) fn boundary(&self) -> &KeyRef {
        &self.boundary
    }

    /// Computes the rank of this [`NodeRef`].
    pub fn rank(&self, factor: u32) -> Rank {
        compute_rank(self.boundary(), factor)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Adoptable for NodeRef {
    async fn adopt<const P: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P>> {
        Node::branch(children, storage).await
    }
}

/// The serializable construct representing a [`Node`].
/// A [`Block`] is what is stored in a [`BlockStore`],
/// used to hydrate and store nodes.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum Block {
    Branch(NonEmpty<NodeRef>),
    Segment(NonEmpty<Entry>),
}

impl Block {
    pub fn branch(data: NonEmpty<NodeRef>) -> Self {
        Block::Branch(data)
    }

    pub fn segment(data: NonEmpty<Entry>) -> Self {
        Block::Segment(data)
    }

    pub fn boundary(&self) -> &KeyRef {
        match self {
            Block::Branch(data) => &data.last().boundary,
            Block::Segment(data) => &data.last().key,
        }
    }

    pub fn node_refs(&self) -> Result<&NonEmpty<NodeRef>> {
        match self {
            Block::Branch(data) => Ok(&data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    pub fn into_node_refs(self) -> Result<NonEmpty<NodeRef>> {
        match self {
            Block::Branch(data) => Ok(data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    pub fn entries(&self) -> Result<&NonEmpty<Entry>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(&data),
        }
    }

    pub fn into_entries(self) -> Result<NonEmpty<Entry>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(data),
        }
    }

    pub async fn encode(&self, storage: &mut impl Storage) -> Result<NodeRef> {
        let hash = storage.write(&self).await?;
        let boundary = self.boundary();
        Ok(NodeRef::new(hash, boundary.to_owned()))
    }

    pub async fn decode(hash: &HashRef, storage: &impl Storage) -> Result<Option<Block>> {
        storage.read(hash).await
    }
}
