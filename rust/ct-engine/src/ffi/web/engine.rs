use crate::{
    ffi::web::{
        cast::{deserialize_js, js_to_string, serialize_js},
        global_initializers,
    },
    Engine, Error,
};
use ct_common::{ModuleDefinition, ModuleId};
use js_sys::Function;
use std::str::FromStr;
use std::{cell::RefCell, rc::Rc};
use tracing::*;
use wasm_bindgen::prelude::*;

/// The [`CtEngine`] constitutes the JavaScript-facing bindings
/// for the Common Runtime.
#[wasm_bindgen(js_name = "CTEngine")]
#[derive(Clone)]
pub struct CtEngine {
    inner: Rc<RefCell<Engine>>,
}

#[wasm_bindgen(js_class = "CTEngine")]
impl CtEngine {
    /// Create a new [`CtEngine`].
    #[wasm_bindgen(constructor)]
    pub fn new(js_callback: Function) -> Self {
        global_initializers();

        let host_callback = move |input: String| {
            let parsed = deserialize_js(&input)?;
            match js_callback.call1(&JsValue::UNDEFINED, &parsed) {
                Ok(js_string) => Ok(serialize_js(&js_string)?),
                Err(js_string) => Err(js_to_string(js_string)?.into()),
            }
        };

        info!("Constructed!");

        Self {
            inner: Rc::new(RefCell::new(
                Engine::new(host_callback)
                    .map_err(|e| format!("Failed to instantiate Common Engine: {e}"))
                    .unwrap(),
            )),
        }
    }

    pub fn define(&mut self, js_definition: JsValue) -> Result<JsValue, JsValue> {
        let definition = ModuleDefinition::from(js_to_string(js_definition)?);
        let module_id = self.inner.borrow_mut().define(definition)?;
        info!("Defining {:?}", module_id);
        Ok(JsValue::from_str(&(module_id.to_string())))
    }

    pub fn run(&mut self, id: JsValue, input: JsValue) -> Result<JsValue, JsValue> {
        let id = ModuleId::from_str(&js_to_string(id)?).map_err(|e| Error::from(e))?;
        let input = serialize_js(&input)?;
        let result = self.inner.borrow_mut().run(&id, input)?;
        Ok(deserialize_js(&result)?)
    }
}
