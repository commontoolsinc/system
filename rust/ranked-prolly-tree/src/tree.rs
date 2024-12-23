use crate::{Adoptable, Entry, EphemeralStorage, Error, HashRef, Key, Node, Result, Storage};
use async_stream::try_stream;
use ct_common::ConditionalSync;
use futures_core::Stream;
use nonempty::NonEmpty;
use std::{collections::BTreeMap, ops::RangeBounds};

#[cfg(doc)]
use crate::Hash;

/// A key-value store backed by a Ranked Prolly Tree with
/// configurable storage and encoding.
#[derive(Clone)]
pub struct Tree<const P: u8, S, K = Vec<u8>, V = Vec<u8>> {
    storage: S,
    root: Option<Node<P, K, V>>,
}

impl<const P: u8, S, K, V> Tree<P, S, K, V>
where
    S: Storage<K, V>,
    K: Key + 'static,
    V: Clone + ConditionalSync,
{
    /// Creates a new [`Tree`] with provided `storage`.
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            root: None,
        }
    }

    /// Hydrate a new [`Tree`] from a node [`Hash`].
    pub async fn from_hash(hash: &HashRef, storage: S) -> Result<Self> {
        let root = Node::from_hash(hash, &storage).await?;
        Ok(Self {
            storage,
            root: Some(root),
        })
    }

    /// Returns a [`Storage`] reference used by this tree.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Returns the [`Node`] representing the root
    /// of this tree.
    ///
    /// Returns `None` if the tree is empty.
    pub fn root(&self) -> Option<&Node<P, K, V>> {
        self.root.as_ref()
    }

    /// Returns the [`Hash`] representing the root
    /// of this tree.
    ///
    /// Returns `None` if the tree is empty.
    pub fn hash(&self) -> Option<&HashRef> {
        self.root().map(|root| root.hash())
    }

    /// Retrieves the value associated with `key` from the tree.
    pub async fn get(&self, key: &K) -> Result<Option<V>> {
        match &self.root {
            Some(root) => match root.get_entry(key, &self.storage).await? {
                Some(entry) => Ok(Some(entry.value)),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// Sets a `key`/`value` pair into the tree.
    pub async fn set(&mut self, key: K, value: V) -> Result<()> {
        let entry = Entry { key, value };
        match &self.root {
            Some(root) => {
                let new_root = root.insert(entry, &mut self.storage).await?;
                self.root = Some(new_root);
            }
            None => {
                let segment = Entry::adopt(NonEmpty::singleton(entry), &mut self.storage).await?;
                self.root = Some(segment);
            }
        }
        Ok(())
    }

    /// Returns an async stream over all entries.
    pub async fn stream<'a>(&'a self) -> impl Stream<Item = Result<Entry<K, V>>> + 'a {
        self.get_range(..).await
    }

    /// Returns an async stream over entries with keys within the provided range.
    pub async fn get_range<'a, R>(
        &'a self,
        range: R,
    ) -> impl Stream<Item = Result<Entry<K, V>>> + 'a
    where
        R: RangeBounds<&'a K> + 'a,
    {
        try_stream! {
            if let Some(root) = self.root.as_ref() {
                let stream = root.get_range(range, &self.storage).await;
                for await item in stream {
                    yield item?;
                }
            }
        }
    }

    /// Create a new [`Tree`] from a [`BTreeMap`].
    ///
    /// A more efficient method than iteratively adding values.
    pub async fn from_set(set: BTreeMap<K, V>, mut storage: S) -> Result<Tree<P, S, K, V>> {
        let mut nodes = {
            let entries = set
                .into_iter()
                .map(|(key, value)| {
                    let node = Entry { key, value };
                    let rank = node.rank(P as u32);
                    (node, rank)
                })
                .collect();
            let entries = NonEmpty::from_vec(entries).ok_or(Error::EmptyChildren)?;
            Node::join_with_rank(entries, 1, &mut storage).await?
        };
        let mut min_rank = 2;
        loop {
            nodes = Node::join_with_rank(nodes, min_rank, &mut storage).await?;
            if nodes.len() == 1 {
                break;
            }
            min_rank += 1;
        }
        Ok(Tree {
            storage,
            root: Some(nodes.head.0),
        })
    }
}

impl<S> From<S> for Tree<64, S, Vec<u8>, Vec<u8>>
where
    S: Storage<Vec<u8>, Vec<u8>>,
{
    fn from(storage: S) -> Self {
        Self::new(storage)
    }
}

impl Default for Tree<64, EphemeralStorage<Vec<u8>, Vec<u8>>, Vec<u8>, Vec<u8>> {
    fn default() -> Self {
        Self::new(EphemeralStorage::default())
    }
}
