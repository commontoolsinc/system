use std::ops::{Bound, RangeBounds};

use crate::{nodes::Block, rank::Rank, Entry, Error, HashRef, NodeRef, Result, Storage};
use async_stream::try_stream;
use async_trait::async_trait;
use futures_core::Stream;
use nonempty::NonEmpty;

#[cfg(doc)]
use crate::Hash;

/// The key type used to store key/value pairs in nodes and trees.
pub type Key = Vec<u8>;
/// A reference to a [`Key`].
pub type KeyRef = <Key as std::ops::Deref>::Target;

/// A helper trait implemented by [`Entry`] and [`NodeRef`] to
/// create new [`Node`]s.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub(crate) trait Adoptable: Sized {
    /// Adopt a collection of `children` into a new [`Node`].
    /// Children data must be ordered and follow rank rules.
    async fn adopt<const P: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P>>;
}

/// Primary representation of tree nodes.
///
/// Each [`Node`] stores its children in a [`Storage`] as key/value pairs.
/// Branches store a collection of children references as [`NodeRef`], and
/// segments (leaf nodes) store their key-value [`Entry`] inline.
#[derive(Clone)]
pub struct Node<const P: u8> {
    pub(crate) block: Block,
    self_ref: NodeRef,
}

impl<const P: u8> Node<P> {
    /// Whether this node is a branch.
    pub fn is_branch(&self) -> bool {
        match self.block {
            Block::Branch(_) => true,
            Block::Segment(_) => false,
        }
    }

    /// Whether this node is a segment, or leaf node.
    pub fn is_segment(&self) -> bool {
        !self.is_branch()
    }

    /// Create a new branch [`Node`] given [`NodeRef`] children,
    /// and encodes the new node into storage.
    pub(crate) async fn branch(
        children: NonEmpty<NodeRef>,
        storage: &mut impl Storage,
    ) -> Result<Self> {
        let block = Block::branch(children);
        let node_ref = block.encode(storage).await?;
        Ok(Node {
            block,
            self_ref: node_ref,
        })
    }

    /// Create a new segment [`Node`] given [`Entry`] children,
    /// and encodes the new node into storage.
    pub(crate) async fn segment(
        children: NonEmpty<Entry>,
        storage: &mut impl Storage,
    ) -> Result<Self> {
        let block = Block::segment(children);
        let node_ref = block.encode(storage).await?;
        Ok(Node {
            block,
            self_ref: node_ref,
        })
    }

    /// Hydrates a node from `storage` given a [`NodeRef`].
    pub async fn from_ref(node_ref: NodeRef, storage: &impl Storage) -> Result<Self> {
        let Some(block) = Block::decode(node_ref.hash(), storage).await? else {
            return Err(Error::MissingBlock(node_ref.hash().to_owned().into()));
        };
        Ok(Node {
            block,
            self_ref: node_ref,
        })
    }

    /// Hydrates a node from `storage` given a [`Hash`].
    pub async fn from_hash(hash: &HashRef, storage: &impl Storage) -> Result<Self> {
        let Some(block) = Block::decode(hash, storage).await? else {
            return Err(Error::MissingBlock(hash.to_owned().into()));
        };
        let node_ref = NodeRef::new(hash.to_owned(), block.boundary().to_owned());
        Ok(Node {
            block,
            self_ref: node_ref,
        })
    }

    /// Returns a [`NodeRef`] for this node.
    pub(crate) fn into_ref(self) -> NodeRef {
        self.self_ref
    }

    /// Returns the [`Hash`] for this node used to retrieve from storage.
    pub fn hash(&self) -> &HashRef {
        &self.self_ref.hash()
    }

    /// Computes the rank of this node.
    pub fn rank(&self) -> Rank {
        self.self_ref.rank(P as u32)
    }

    /// Return all entries from this node into a [`Entry`] collection.
    ///
    /// Returns an error if this is not a segment node.
    pub fn into_entries(self) -> Result<NonEmpty<Entry>> {
        if !self.is_segment() {
            return Err(Error::SegmentOnly);
        }

        self.block.into_entries()
    }

    /// Recursively descends the tree, returning an [`Entry`] matching
    /// `key` if found.
    pub async fn get_entry(&self, key: &KeyRef, storage: &impl Storage) -> Result<Option<Entry>> {
        #[allow(unused_assignments)]
        let mut current_node_holder: Option<Node<P>> = None;
        let mut current_node = self;
        loop {
            match current_node.is_branch() {
                true => {
                    let Some(node) = current_node.child_by_key(key, storage).await? else {
                        return Ok(None);
                    };
                    current_node_holder = Some(node);
                    current_node = current_node_holder.as_ref().unwrap();
                }
                false => return current_node.entry_by_key(key),
            }
        }
    }

    /// Returns an async stream over entries with keys within the provided range.
    pub async fn get_range<'a, R>(
        &'a self,
        range: R,
        storage: &'a impl Storage,
    ) -> impl Stream<Item = Result<Entry>> + 'a
    where
        R: RangeBounds<&'a KeyRef> + 'a,
    {
        async fn get_child_index_by_key<const P: u8>(
            node: &Node<P>,
            key: &KeyRef,
            storage: &impl Storage,
        ) -> Result<Option<(Node<P>, usize)>> {
            for (index, node_ref) in node.block.node_refs()?.iter().enumerate() {
                if *key <= *node_ref.boundary() {
                    return Ok(Some((
                        Node::from_ref(node_ref.to_owned(), storage).await?,
                        index,
                    )));
                }
            }
            Ok(None)
        }

        struct Level<const P: u8> {
            node: Node<P>,
            visited_index: Option<usize>,
        }
        impl<const P: u8> Level<P> {
            fn new(node: Node<P>, visited_index: Option<usize>) -> Self {
                Level {
                    node,
                    visited_index,
                }
            }
        }

        // Get the start key. Included/Excluded ranges are identical here,
        // the check if key is in range is below, and this will at most read
        // one unnecessary segment iff `Bound::Excluded(K)` and `K` is a boundary node.
        const UNBOUNDED_START_KEY: [u8; 1] = [0];
        let start_key = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start,
            Bound::Unbounded => &UNBOUNDED_START_KEY,
        };
        // An entry was found matching the key range.
        let mut matching = false;

        // Track ancestor nodes and the index of the most recently visited child
        let mut branch_stack = vec![Level::new(self.to_owned(), None)];
        try_stream! {
            loop {
                let Some(current) = branch_stack.last_mut() else {
                    return;
                };
                match current.node.is_branch() {
                    true => {
                        if !matching {
                            let Some((next_node, next_index)) = get_child_index_by_key(&current.node, start_key, storage).await? else {
                                // The start key is larger than any key stored in this tree.
                                return;
                            };
                            current.visited_index = Some(next_index);
                            branch_stack.push(Level::new(next_node, None));
                        } else {
                            let next_index = match current.visited_index {
                                Some(visited_index) => visited_index + 1,
                                None => 0
                            };
                            match current.node.block.node_refs()?.get(next_index) {
                                Some(node_ref) => {
                                    let next_node = Node::from_ref(node_ref.to_owned(), storage).await?;
                                    current.visited_index = Some(next_index);
                                    branch_stack.push(Level::new(next_node, None));
                                }
                                None => {
                                    // Parent needs to check next sibling
                                    branch_stack.pop();
                                }
                            }
                        }
                    }
                    false => {
                        let current = branch_stack.pop().ok_or(Error::Unexpected)?;
                        for entry in current.node.into_entries()? {
                            let entry_key: &[u8] = entry.key.as_ref();
                            if range.contains(&entry_key) {
                                if !matching {
                                    matching = true;
                                }
                                yield entry;
                            } else if matching {
                                // We've surpassed the range; abort.
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Inserts a new [`Entry`] into the tree represented by this node as root.
    /// On success, returns the new root [`Node`] representing this tree.
    pub async fn insert(&self, new_entry: Entry, storage: &mut impl Storage) -> Result<Node<P>> {
        let key = new_entry.key.to_owned();
        let mut node = self.to_owned();
        let mut branch_stack = vec![];
        #[allow(unused_assignments)]
        let mut all_entries: Option<NonEmpty<Entry>> = None;
        loop {
            match node.is_branch() {
                true => {
                    let mut left = vec![];
                    let mut right = vec![];
                    let mut next = None;
                    for child_ref in node.block.into_node_refs()? {
                        // If key may be contained within the child ref, or if it's
                        // the largest boundary use the last child.
                        if next.is_some() {
                            right.push(child_ref);
                        } else if *key <= *child_ref.boundary() {
                            next = Some(Node::from_ref(child_ref, storage).await?);
                        } else {
                            left.push(child_ref);
                        }
                    }
                    // If key is greater than the greatest child, use the
                    // greatest child.
                    if next.is_none() {
                        let last = left.pop().ok_or(Error::Unexpected)?;
                        next = Some(Node::from_ref(last, storage).await?);
                    }
                    branch_stack.push((NonEmpty::from_vec(left), NonEmpty::from_vec(right)));
                    node = next.ok_or(Error::Unexpected)?;
                }
                false => {
                    let mut entries = node.block.into_entries()?;
                    match entries.binary_search_by(|probe| probe.key.cmp(&key)) {
                        // Entry was found; update the value.
                        Ok(index) => {
                            let Some(previous_entry) = entries.get_mut(index) else {
                                return Err(Error::Unexpected);
                            };
                            previous_entry.value = new_entry.value;
                        }
                        // Entry was not found; insert at the provided index.
                        Err(index) => {
                            entries.insert(index, new_entry);
                        }
                    };
                    all_entries = Some(entries);
                    break;
                }
            }
        }
        let mut nodes = {
            let Some(entries) = all_entries else {
                return Err(Error::Unexpected);
            };
            let entries = entries.map(|entry| {
                let rank = entry.rank(P as u32);
                (entry, rank)
            });
            Node::join_with_rank(entries, 1, storage).await?
        };
        let mut min_rank = 2;
        loop {
            let node_refs = {
                let node_refs = nodes.map(|(node, rank)| (node.into_ref(), rank));
                match branch_stack.pop() {
                    Some(siblings) => {
                        // TBD if we must recompute rank for siblings references
                        // when building up the tree.
                        // Attempt to try setting rank to `0` for node refs outside
                        // of the modified path.
                        let left = siblings.0.map(|left| {
                            left.map(|node_ref| {
                                let rank = node_ref.rank(P as u32);
                                (node_ref, rank)
                            })
                        });
                        let right = siblings.1.map(|right| {
                            right.map(|node_ref| {
                                let rank = node_ref.rank(P as u32);
                                (node_ref, rank)
                            })
                        });
                        match (left, right) {
                            (None, None) => node_refs,
                            (Some(left), None) => concat_nonempty(vec![left, node_refs])?,
                            (None, Some(right)) => concat_nonempty(vec![node_refs, right])?,
                            (Some(left), Some(right)) => {
                                concat_nonempty(vec![left, node_refs, right])?
                            }
                        }
                    }
                    None => node_refs,
                }
            };

            nodes = Node::join_with_rank(node_refs, min_rank, storage).await?;
            if branch_stack.is_empty() && nodes.len() == 1 {
                break;
            }
            min_rank += 1;
        }
        Ok(nodes.head.0)
    }

    /// Returns the decoded child [`Node`] that may contain `key`
    /// within its descendants.
    ///
    /// Returns an error if this is not a branch node.
    async fn child_by_key(&self, key: &KeyRef, storage: &impl Storage) -> Result<Option<Node<P>>> {
        if !self.is_branch() {
            return Err(Error::BranchOnly);
        }
        for node_ref in self.block.node_refs()? {
            if *key <= *node_ref.boundary() {
                return Ok(Some(Node::from_ref(node_ref.to_owned(), storage).await?));
            }
        }
        Ok(None)
    }

    /// Returns this segment's [`Entry`] matching the provided `key`.
    ///
    /// Returns an error if this is not a segment node.
    fn entry_by_key(&self, key: &KeyRef) -> Result<Option<Entry>> {
        if !self.is_segment() {
            return Err(Error::SegmentOnly);
        }
        for entry in self.block.entries()? {
            if *key == *entry.key {
                return Ok(Some(entry.to_owned()));
            }
        }
        Ok(None)
    }

    /// Joins a collection of sibling [`Adoptable`]s into
    /// one or more parent [`Node`]s, where branching is determined
    /// by rank.
    pub(crate) async fn join_with_rank<T: Adoptable>(
        nodes: NonEmpty<(T, Rank)>,
        min_rank: Rank,
        storage: &mut impl Storage,
    ) -> Result<NonEmpty<(Node<P>, Rank)>> {
        let mut output = vec![];
        let mut pending = vec![];
        for (node, rank) in nodes {
            pending.push(node);
            if rank > min_rank {
                let children = NonEmpty::from_vec(std::mem::replace(&mut pending, vec![]))
                    .ok_or(Error::Unexpected)?;
                let node = T::adopt(children, storage).await?;
                output.push((node, rank));
            }
        }
        if let Some(pending) = NonEmpty::from_vec(pending) {
            let node = T::adopt(pending, storage).await?;
            output.push((node, min_rank));
        }
        NonEmpty::from_vec(output).ok_or(Error::Unexpected)
    }
    /*
       pub(crate) async fn zip(
           entry: Option<Entry>,
           stack: UnzipStack,
           storage: &mut impl Storage,
       ) -> Result<Node<P>> {
           let mut nodes = {
               let Some(entries) = all_entries else {
                   return Err(Error::Unexpected);
               };
               let entries = entries.map(|entry| {
                   let rank = entry.rank(P as u32);
                   (entry, rank)
               });
               Node::join_with_rank(entries, 1, storage).await?
           };
           let mut min_rank = 2;
           loop {
               let node_refs = {
                   let node_refs = nodes.map(|(node, rank)| (node.into_ref(), rank));
                   match branch_stack.pop() {
                       Some(siblings) => {
                           // TBD if we must recompute rank for siblings references
                           // when building up the tree.
                           // Attempt to try setting rank to `0` for node refs outside
                           // of the modified path.
                           let left = siblings.0.map(|left| {
                               left.map(|node_ref| {
                                   let rank = node_ref.rank(P as u32);
                                   (node_ref, rank)
                               })
                           });
                           let right = siblings.1.map(|right| {
                               right.map(|node_ref| {
                                   let rank = node_ref.rank(P as u32);
                                   (node_ref, rank)
                               })
                           });
                           match (left, right) {
                               (None, None) => node_refs,
                               (Some(left), None) => concat_nonempty(vec![left, node_refs])?,
                               (None, Some(right)) => concat_nonempty(vec![node_refs, right])?,
                               (Some(left), Some(right)) => {
                                   concat_nonempty(vec![left, node_refs, right])?
                               }
                           }
                       }
                       None => node_refs,
                   }
               };

               nodes = Node::join_with_rank(node_refs, min_rank, storage).await?;
               if branch_stack.is_empty() && nodes.len() == 1 {
                   break;
               }
               min_rank += 1;
           }
       }
    */
}

/*
pub(crate) struct UnzipStack(Vec<(Vec<NodeFragment>, Vec<NodeFragment>)>);
impl UnzipStack {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, item: (Vec<NodeFragment>, Vec<NodeFragment>)) {
        self.0.push(item)
    }

    fn pop(&mut self) -> Option<(Vec<NodeFragment>, Vec<NodeFragment>)> {
        self.0.pop()
    }
}
pub(crate) enum NodeFragment {
    NodeRef(NodeRef),
    Entry(Entry),
}

#[async_trait]
impl Adoptable for Entry {
    async fn adopt<const P: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P>> {
        Node::segment(children, storage).await
    }
}

#[async_trait]
impl Adoptable for NodeRef {
    async fn adopt<const P: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P>> {
        Node::branch(children, storage).await
    }
}
impl NodeFragment {
    async fn adopt<const P: u8>(children: NonEmpty<Self>, storage: &mut impl Storage) -> Result<Node<P, E>> {
        match children.head {
            NodeFragment::Entry()
        }
    }
}

impl From<Entry> for NodeFragment {
    fn from(value: Entry) -> Self {
        NodeFragment::Entry(value)
    }
}

impl From<NodeRef> for NodeFragment {
    fn from(value: NodeRef) -> Self {
        NodeFragment::NodeRef(value)
    }
}
*/

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8> Adoptable for Node<P> {
    async fn adopt<const P2: u8>(
        children: NonEmpty<Self>,
        storage: &mut impl Storage,
    ) -> Result<Node<P2>> {
        Node::branch(children.map(|node| node.into_ref()), storage).await
    }
}

/// TODO: Improve. Possibly remove NonEmpty as it introduces
/// some overhead compared to index comparison with slices.
fn concat_nonempty<T>(list: Vec<NonEmpty<T>>) -> Result<NonEmpty<T>> {
    Ok(NonEmpty::flatten(
        NonEmpty::from_vec(list).ok_or(Error::EmptyChildren)?,
    ))
}
