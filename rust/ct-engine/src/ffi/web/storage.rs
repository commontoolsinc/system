use crate::{
    ffi::web::{cast::bytes_to_typed_array, global_initializers},
    Error, Result,
};
use ct_common::{ModuleDefinition, ModuleId};
use ct_storage::{CtStorage, Key, PlatformStorage};
use futures_util::TryStreamExt;
use std::{cell::RefCell, rc::Rc};
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

    /// Sets the key from the component parts to `value`.
    pub async fn set(
        &mut self,
        entity: String,
        ns: String,
        attr: String,
        value: Box<[u8]>,
    ) -> Result<()> {
        let key = Key::new(&entity, &ns, &attr);
        let value: Vec<u8> = value.into();
        self.inner
            .borrow_mut()
            .set(key, value)
            .await
            .map_err(|e| e.into())
    }

    /// Sets `key` to `value`.
    #[wasm_bindgen(js_name = "setByKey")]
    pub async fn set_by_key(&mut self, key: Box<[u8]>, value: Box<[u8]>) -> Result<()> {
        let key = Key::try_from(key.into_vec()).map_err(|e| Error::from(e))?;
        let value: Vec<u8> = value.into();
        self.inner
            .borrow_mut()
            .set(key, value)
            .await
            .map_err(|e| e.into())
    }

    /// Gets a value for the key comprised of component parts.
    pub async fn get(
        &mut self,
        entity: String,
        ns: String,
        attr: String,
    ) -> Result<Option<Vec<u8>>> {
        let key = Key::new(&entity, &ns, &attr);
        self.inner
            .borrow_mut()
            .get(&key)
            .await
            .map_err(|e| e.into())
    }

    /// Retrieves value with `key`.
    #[wasm_bindgen(js_name = "getByKey")]
    pub async fn get_by_key(&self, key: Box<[u8]>) -> Result<Option<Vec<u8>>> {
        let key = Key::try_from(key.into_vec()).map_err(|e| Error::from(e))?;
        self.inner.borrow().get(&key).await.map_err(|e| e.into())
    }

    /// Calls `callback` with `key` and `value` arguments
    /// for each entry within `start` and `end` (inclusive) range.
    #[wasm_bindgen(js_name = "getRange")]
    pub async fn get_range(
        &self,
        start_entity: String,
        start_ns: String,
        start_attr: String,
        end_entity: String,
        end_ns: String,
        end_attr: String,
        callback: &js_sys::Function,
    ) -> Result<()> {
        let this = JsValue::null();
        let start = Key::new(&start_entity, &start_ns, &start_attr);
        let end = Key::new(&end_entity, &end_ns, &end_attr);
        let inner = self.inner.borrow();
        let stream = inner.get_range(&start..=&end).await;
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

    /// Calls `callback` with `key` and `value` arguments
    /// for each entry within `start` and `end` (inclusive) range.
    #[wasm_bindgen(js_name = "getRangeByKey")]
    pub async fn get_range_by_key(
        &self,
        start: Box<[u8]>,
        end: Box<[u8]>,
        callback: &js_sys::Function,
    ) -> Result<()> {
        let this = JsValue::null();
        let start = Key::try_from(start.into_vec()).map_err(|e| Error::from(e))?;
        let end = Key::try_from(end.into_vec()).map_err(|e| Error::from(e))?;
        let inner = self.inner.borrow();
        let stream = inner.get_range(&start..=&end).await;
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
