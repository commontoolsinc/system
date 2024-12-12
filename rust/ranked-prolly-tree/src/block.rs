use crate::{compute_rank, Adoptable, Error, Hash, HashRef, Key, Node, Rank, Result, Storage};
use async_trait::async_trait;
use ct_common::ConditionalSync;
use nonempty::NonEmpty;

/// A key-value entry in a tree.
#[derive(Debug, PartialEq)]
pub struct Entry<K, V> {
    /// The key in this key/value pair.
    pub key: K,
    /// The value in this key/value pair.
    pub value: V,
}

impl<K, V> Entry<K, V>
where
    K: AsRef<[u8]>,
{
    /// Create a new [`Entry`].
    pub fn new(key: K, value: V) -> Self {
        Entry { key, value }
    }

    /// Computes the rank of the [`Entry`]'s key.
    pub fn rank(&self, factor: u32) -> Rank {
        compute_rank(self.key.as_ref(), factor)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, K, V> Adoptable<P, K, V> for Entry<K, V>
where
    K: Key + 'static,
    V: Clone + ConditionalSync,
{
    async fn adopt(
        children: NonEmpty<Self>,
        storage: &mut impl Storage<K, V>,
    ) -> Result<Node<P, K, V>> {
        Node::segment(children, storage).await
    }
}

impl<K, V> Clone for Entry<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

/// A serializable reference to a [`Node`].
#[derive(Debug, PartialEq)]
pub struct NodeRef<K, H> {
    boundary: K,
    hash: H,
}

impl<K, H> NodeRef<K, H>
where
    K: Key,
{
    /// Create a new [`NodeRef`].
    pub fn new(boundary: K, hash: H) -> Self {
        NodeRef { hash, boundary }
    }

    /// The hash for this [`NodeRef`].
    pub fn hash(&self) -> &H {
        &self.hash
    }

    /// The upper bounds as a key for this [`NodeRef`].
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
impl<const P: u8, K, V> Adoptable<P, K, V> for NodeRef<K, Hash>
where
    K: Key + 'static,
    V: Clone + ConditionalSync,
{
    async fn adopt(
        children: NonEmpty<Self>,
        storage: &mut impl Storage<K, V>,
    ) -> Result<Node<P, K, V>> {
        Node::branch(children, storage).await
    }
}

impl<K, H> Clone for NodeRef<K, H>
where
    K: Clone,
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            boundary: self.boundary.clone(),
            hash: self.hash.clone(),
        }
    }
}

/// The serializable construct representing a [`Node`].
/// A [`Block`] is what is stored in a [`BlockStore`],
/// used to hydrate and store nodes.
#[derive(Debug, PartialEq)]
pub enum Block<K, V> {
    /// A block representing a Branch.
    Branch(NonEmpty<NodeRef<K, Vec<u8>>>),
    /// A block representing a Segment.
    Segment(NonEmpty<Entry<K, V>>),
}

impl<K, V> Block<K, V>
where
    K: Key + 'static,
    V: ConditionalSync,
{
    /// Create a new branch-type block.
    pub fn branch(data: NonEmpty<NodeRef<K, Hash>>) -> Self {
        Block::Branch(data)
    }

    /// Create a new segment-type block.
    pub fn segment(data: NonEmpty<Entry<K, V>>) -> Self {
        Block::Segment(data)
    }

    /// Whether this block is a branch.
    pub fn is_branch(&self) -> bool {
        match self {
            Block::Branch(_) => true,
            _ => false,
        }
    }

    /// Whether this block is a segment.
    pub fn is_segment(&self) -> bool {
        !self.is_branch()
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
    pub fn node_refs(&self) -> Result<&NonEmpty<NodeRef<K, Hash>>> {
        match self {
            Block::Branch(data) => Ok(&data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    /// Takes children data as [`NodeRef`]s.
    ///
    /// Returns an `Err` if a segment-type.
    pub fn into_node_refs(self) -> Result<NonEmpty<NodeRef<K, Hash>>> {
        match self {
            Block::Branch(data) => Ok(data),
            Block::Segment(_) => Err(Error::BranchOnly),
        }
    }

    /// Get children data as [`Entry`]s.
    ///
    /// Returns an `Err` if a branch-type.
    pub fn entries(&self) -> Result<&NonEmpty<Entry<K, V>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(&data),
        }
    }

    /// Take children data as [`Entry`]s.
    ///
    /// Returns an `Err` if a branch-type.
    pub fn into_entries(self) -> Result<NonEmpty<Entry<K, V>>> {
        match self {
            Block::Branch(_) => Err(Error::SegmentOnly),
            Block::Segment(data) => Ok(data),
        }
    }

    /// Write block into `storage`.
    pub async fn encode(&self, storage: &mut impl Storage<K, V>) -> Result<NodeRef<K, Hash>> {
        let hash = storage.write(&self).await?;
        let boundary = self.boundary();
        Ok(NodeRef::new(boundary.to_owned(), hash))
    }

    /// Read block from `storage` given a hash.
    pub async fn decode(
        hash: &HashRef,
        storage: &impl Storage<K, V>,
    ) -> Result<Option<Block<K, V>>> {
        storage.read(hash).await
    }
}

impl<K, V> Clone for Block<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Block::Branch(inner) => Block::Branch(inner.to_owned()),
            Block::Segment(inner) => Block::Segment(inner.to_owned()),
        }
    }
}
