use crate::{
    ffi::web::{
        cast::{deserialize_js, js_to_string, serialize_js},
        global_initializers,
    },
    Error, Result,
};
use ct_common::{ModuleDefinition, ModuleId};
use ct_runtime::{Runtime, VirtualMachine};
use js_sys::Function;
use ranked_prolly_tree::{
    BincodeEncoder, HashDisplay, IndexedDbStore, LruStore, NodeStorage, Tree,
};
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};
use tracing::*;
use wasm_bindgen::prelude::*;

/// LRU cache size.
const LRU_CACHE_SIZE: usize = 10_000;
/// Branching factor.
const P: u8 = 64;

type WebStorage = NodeStorage<BincodeEncoder, LruStore<IndexedDbStore>>;

/// The [`CTStore`] provides direct access to the underlying web store.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CTStore {
    inner: Rc<RefCell<Tree<P, WebStorage>>>,
}

#[wasm_bindgen]
impl CTStore {
    /// Create a new [`CTStore`].
    #[wasm_bindgen(constructor)]
    pub async fn new(db_name: String, store_name: String, hash: Option<Box<[u8]>>) -> Result<Self> {
        global_initializers();
        info!("Constructed!");

        let store = IndexedDbStore::new(&db_name, &store_name).await?;
        let store = LruStore::new(store, LRU_CACHE_SIZE)?;
        let storage = NodeStorage::new(BincodeEncoder::default(), store);
        let tree = match hash {
            Some(hash) => Tree::from_hash(&hash, storage).await?,
            None => Tree::new(storage),
        };
        Ok(Self {
            inner: Rc::new(RefCell::new(tree)),
        })
    }

    /// Returns the root hash of the storage, if it contains data.
    pub fn hash(&self) -> Option<Box<[u8]>> {
        self.inner
            .borrow()
            .hash()
            .map(|hash| hash.to_owned().into())
    }

    /// Sets `key` to `value`.
    pub async fn set(&mut self, key: Box<[u8]>, value: Box<[u8]>) -> Result<()> {
        let key: Vec<u8> = key.into();
        let value: Vec<u8> = value.into();
        self.inner
            .borrow_mut()
            .set(key, value)
            .await
            .map_err(|e| e.into())
    }

    /// Retrieves value with `key`.
    pub async fn get(&self, key: Box<[u8]>) -> Result<Option<Vec<u8>>> {
        self.inner.borrow().get(&key).await.map_err(|e| e.into())
    }
}
