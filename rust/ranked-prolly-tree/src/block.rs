use crate::{compute_rank, Adoptable, Error, Hash, HashRef, Key, Node, Rank, Result, Storage};
use async_trait::async_trait;
use nonempty::NonEmpty;
use serde::{Deserialize, Serialize};

/// A key-value entry in a tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    K: Key + 'static,
{
    async fn adopt(children: NonEmpty<Self>, storage: &mut impl Storage<K>) -> Result<Node<P, K>> {
        Node::segment(children, storage).await
    }
}

/// A serializable reference to a [`Node`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeRef<K> {
    boundary: K,
    hash: Hash,
}

impl<K> NodeRef<K>
where
    K: Key,
{
    pub fn new(boundary: K, hash: Hash) -> Self {
        NodeRef { hash, boundary }
    }

    pub fn hash(&self) -> &HashRef {
        &self.hash
    }

    pub fn boundary(&self) -> &K {
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
    K: Key + 'static,
{
    async fn adopt(children: NonEmpty<Self>, storage: &mut impl Storage<K>) -> Result<Node<P, K>> {
        Node::branch(children, storage).await
    }
}

/// The serializable construct representing a [`Node`].
/// A [`Block`] is what is stored in a [`BlockStore`],
/// used to hydrate and store nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block<K> {
    /// A block representing a Branch.
    Branch(NonEmpty<NodeRef<K>>),
    /// A block representing a Segment.
    Segment(NonEmpty<Entry<K>>),
}

impl<K> Block<K>
where
    K: Key,
{
    /// Create a new branch-type block.
    pub fn branch(data: NonEmpty<NodeRef<K>>) -> Self {
        Block::Branch(data)
    }

    /// Create a new segment-type block.
    pub fn segment(data: NonEmpty<Entry<K>>) -> Self {
        Block::Segment(data)
    }

    /// Get the upper bounds that this block represents.
    pub fn boundary(&self) -> &K {
        match self {
            Block::Branch(data) => &data.last().boundary,
            Block::Segment(data) => &data.last().key,
        }
    }

    /// Get children data as [`NodeRef`]s.
    ///
    /// Returns an `Err` if a segment-type.
    pub fn node_refs(&self) -> Result<&NonEmpty<NodeRef<K>>> {
        match self {
            Block::Branch(data) => Ok(&data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    /// Takes children data as [`NodeRef`]s.
    ///
    /// Returns an `Err` if a segment-type.
    pub fn into_node_refs(self) -> Result<NonEmpty<NodeRef<K>>> {
        match self {
            Block::Branch(data) => Ok(data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    /// Get children data as [`Entry`]s.
    ///
    /// Returns an `Err` if a branch-type.
    pub fn entries(&self) -> Result<&NonEmpty<Entry<K>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(&data),
        }
    }

    /// Take children data as [`Entry`]s.
    ///
    /// Returns an `Err` if a branch-type.
    pub fn into_entries(self) -> Result<NonEmpty<Entry<K>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(data),
        }
    }

    /// Returns children as key value pairs generically.
    /// TODO: Use same inner types.
    pub fn key_values(&self) -> Vec<(&K, &[u8])> {
        match self {
            Block::Branch(data) => data
                .iter()
                .map(|node_ref| (&node_ref.boundary, node_ref.hash.as_slice()))
                .collect(),
            Block::Segment(data) => data
                .iter()
                .map(|entry| (&entry.key, entry.value.as_slice()))
                .collect(),
        }
    }

    /// Write block into `storage`.
    pub async fn encode(&self, storage: &mut impl Storage<K>) -> Result<NodeRef<K>>
    where
        K: 'static,
    {
        let hash = storage.write(&self).await?;
        let boundary = self.boundary();
        Ok(NodeRef::new(boundary.to_owned(), hash))
    }

    /// Read block from `storage` given a hash.
    pub async fn decode(hash: &HashRef, storage: &impl Storage<K>) -> Result<Option<Block<K>>> {
        storage.read(hash).await
    }
}
