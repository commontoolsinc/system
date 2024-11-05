use crate::{
    ffi::web::{
        cast::{bytes_to_typed_array, deserialize_js, js_to_string, serialize_js},
        global_initializers,
    },
    Error, Result,
};
use ct_common::{ModuleDefinition, ModuleId};
use ct_runtime::{Runtime, VirtualMachine};
use ct_storage::Storage;
use futures_util::TryStreamExt;
use js_sys::Function;
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};
use tracing::*;
use wasm_bindgen::prelude::*;

/// The [`CTStore`] provides direct access to the underlying web store.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CTStore {
    inner: Rc<RefCell<Storage>>,
}

#[wasm_bindgen]
impl CTStore {
    /// Create a new [`CTStore`].
    #[wasm_bindgen(constructor)]
    pub async fn new(db_name: String, store_name: String, hash: Option<Box<[u8]>>) -> Result<Self> {
        global_initializers();
        info!("Constructed!");

        let storage = Storage::open((db_name, store_name), hash.map(|hash| hash.to_vec())).await?;
        Ok(Self {
            inner: Rc::new(RefCell::new(storage)),
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

    /// Calls `callback` with `key` and `value` arguments
    /// for each entry within `start` and `end` (inclusive) range.
    #[wasm_bindgen(js_name = "getRange")]
    pub async fn get_range(
        &self,
        start: Box<[u8]>,
        end: Box<[u8]>,
        callback: &js_sys::Function,
    ) -> Result<()> {
        let this = JsValue::null();
        let inner = self.inner.borrow();
        let stream = inner.get_range(start.as_ref()..=end.as_ref()).await;
        tokio::pin!(stream);
        while let Some(entry) = stream.try_next().await? {
            callback.call2(
                &this,
                &bytes_to_typed_array(&entry.key)?,
                &bytes_to_typed_array(&entry.value)?,
            );
        }
        Ok(())
    }
}
