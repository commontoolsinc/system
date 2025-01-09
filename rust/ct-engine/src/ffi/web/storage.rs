use crate::{
    ffi::web::{cast::bytes_to_typed_array, global_initializers},
    Error, Result,
};
use ct_common::{ModuleDefinition, ModuleId};
use ct_storage::{CtStorage, Key, PlatformStorage};
use futures_util::TryStreamExt;
use std::{cell::RefCell, ops::Bound, rc::Rc};
use tracing::*;
use wasm_bindgen::prelude::*;

/// The [`CtStore`] provides direct access to the underlying web store.
#[wasm_bindgen(js_name = "CTStore")]
#[derive(Clone)]
pub struct CtStore {
    inner: Rc<RefCell<CtStorage<PlatformStorage>>>,
}

#[wasm_bindgen(js_class = "CTStore")]
impl CtStore {
    /// Create a new [`CtStore`].
    #[wasm_bindgen(constructor)]
    pub async fn new(db_name: String, store_name: String, hash: Option<Box<[u8]>>) -> Result<Self> {
        global_initializers();
        info!("Constructed!");

        let storage = CtStorage::<PlatformStorage>::open_idb(
            db_name,
            store_name,
            hash.map(|hash| hash.to_vec()),
        )
        .await?;
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
    #[wasm_bindgen(js_name = "set")]
    pub async fn set(&mut self, key: Box<[u8]>, value: Box<[u8]>) -> Result<()> {
        let key = Key::try_from(key.into_vec()).map_err(|e| Error::from(e))?;
        let value: Vec<u8> = value.into();
        self.inner
            .borrow_mut()
            .set(key, value)
            .await
            .map_err(|e| e.into())
    }

    /// Retrieves value with `key`.
    #[wasm_bindgen(js_name = "get")]
    pub async fn get(&self, key: Box<[u8]>) -> Result<Option<Vec<u8>>> {
        let key = Key::try_from(key.into_vec()).map_err(|e| Error::from(e))?;
        self.inner.borrow().get(&key).await.map_err(|e| e.into())
    }

    /// Calls `callback` with `key` and `value` arguments
    /// for each entry within `start` and `end` range.
    #[wasm_bindgen(js_name = "getRange")]
    pub async fn get_range(
        &self,
        start: Box<[u8]>,
        end: Box<[u8]>,
        start_inclusive: bool,
        end_inclusive: bool,
        callback: &js_sys::Function,
    ) -> Result<()> {
        let this = JsValue::null();
        let start = Key::try_from(start.into_vec()).map_err(|e| Error::from(e))?;
        let end = Key::try_from(end.into_vec()).map_err(|e| Error::from(e))?;
        let range = match (start_inclusive, end_inclusive) {
            (true, true) => (Bound::Included(start), Bound::Included(end)),
            (true, false) => (Bound::Included(start), Bound::Excluded(end)),
            (false, true) => (Bound::Excluded(start), Bound::Included(end)),
            (false, false) => (Bound::Excluded(start), Bound::Excluded(end)),
        };
        let inner = self.inner.borrow();
        let stream = inner.stream_range(range).await;
        tokio::pin!(stream);
        while let Some(entry) = stream.try_next().await? {
            callback.call2(
                &this,
                &bytes_to_typed_array(entry.key.as_ref())?,
                &bytes_to_typed_array(entry.value.as_ref())?,
            )?;
        }
        Ok(())
    }
}
