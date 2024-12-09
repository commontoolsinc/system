use crate::{BlockStore, Error, Hash, HashRef, Result};
use async_trait::async_trait;
use js_sys::Uint8Array;
use rexie::{ObjectStore, Rexie, RexieBuilder, TransactionMode};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};

const INDEXEDDB_STORAGE_VERSION: u32 = 1;

/// An IndexedDb [`BlockStore`] implementation.
#[derive(Clone)]
pub struct IndexedDbStore {
    db: Rc<Rexie>,
    store_name: String,
}

impl IndexedDbStore {
    /// Creates a new [`IndexedDbStore`].
    pub async fn new(db_name: &str, store_name: &str) -> Result<Self> {
        let db = RexieBuilder::new(db_name)
            .version(INDEXEDDB_STORAGE_VERSION)
            .add_object_store(ObjectStore::new(store_name).auto_increment(false))
            .build()
            .await
            .map_err(|error| Error::Io(error.to_string()))?;

        Ok(IndexedDbStore {
            db: Rc::new(db),
            store_name: store_name.to_owned(),
        })
    }
}

#[async_trait(?Send)]
impl BlockStore for IndexedDbStore {
    async fn get_block(&self, hash: &HashRef) -> Result<Option<Vec<u8>>> {
        let tx = self
            .db
            .transaction(&[&self.store_name], TransactionMode::ReadOnly)?;
        let store = tx.store(&self.store_name)?;
        let key = bytes_to_typed_array(hash)?;
        let Some(value) = store.get(key).await? else {
            return Ok(None);
        };
        let out = value.dyn_into::<Uint8Array>()?.to_vec();
        tx.done().await?;
        Ok(Some(out))
    }

    async fn set_block(&mut self, hash: Hash, bytes: Vec<u8>) -> Result<()> {
        let tx = self
            .db
            .transaction(&[&self.store_name], TransactionMode::ReadWrite)?;
        let store = tx.store(&self.store_name)?;
        let key = bytes_to_typed_array(&hash)?;
        let value = bytes_to_typed_array(&bytes)?;
        store.put(&value, Some(&key)).await?;
        tx.done().await?;
        Ok(())
    }
}

fn bytes_to_typed_array(bytes: &[u8]) -> Result<JsValue> {
    let array = Uint8Array::new_with_length(bytes.len() as u32);
    array.copy_from(&bytes);
    Ok(JsValue::from(array))
}
