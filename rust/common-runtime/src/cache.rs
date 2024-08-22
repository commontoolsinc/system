use std::{borrow::Borrow, sync::Arc};

use sieve_cache::SieveCache;
use std::hash::Hash;
use tokio::sync::Mutex;

use crate::CommonRuntimeError;

/// A general purpose cache for Runtime internals, built around a [SieveCache]
/// See also: https://cachemon.github.io/SIEVE-website/
///
/// [Cache] is cheaply cloneable and threadsafe.
#[derive(Clone)]
pub struct Cache<K, V>(Arc<Mutex<SieveCache<K, V>>>)
where
    K: Eq + Hash + Clone,
    V: Clone;

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Instantiate a new [Cache] with a given capacity
    pub fn new(capacity: usize) -> Result<Self, CommonRuntimeError> {
        Ok(Cache(Arc::new(Mutex::new(
            SieveCache::new(capacity)
                .map_err(|error| CommonRuntimeError::InternalError(error.into()))?,
        ))))
    }

    /// Look up an item that may be in the [Cache]
    pub async fn get<Q>(&self, key: &Q) -> Option<V>
    where
        Q: Hash + Eq + ?Sized,
        K: Borrow<Q>,
    {
        let mut sieve = self.0.lock().await;
        sieve.get(key).cloned()
    }

    /// Add an item to the [Cache], evicting older items if the [Cache] is
    /// already at capacity
    pub async fn insert(&self, key: K, value: V) {
        let mut sieve = self.0.lock().await;
        sieve.insert(key, value);
    }
}
