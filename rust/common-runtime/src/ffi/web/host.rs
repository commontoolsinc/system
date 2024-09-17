use std::collections::BTreeMap;

use wasm_bindgen::prelude::*;
use web_sys::js_sys::{Object, Reflect};

use crate::{ffi::Value, IoValues, Value as RuntimeValue};

// https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/structural.html

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_DEFINITIONS: &'static str = r#"
type JavaScriptValueMap = {
    [index: string]: JavaScriptValue
}

type JavaScriptShapeMap = {
    [index: string]: "string"|"boolean"|"number"|"buffer"
}

interface JavaScriptValue {
    tag: string;
    val: string|number|boolean|Uint8Array;
}

interface JavaScriptModuleDefinition {
    inputs: JavaScriptValueMap;
    outputs: JavaScriptShapeMap;
    body: string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "JavaScriptValueMap")]
    pub type JavaScriptValueMap;

    #[wasm_bindgen(typescript_type = "JavaScriptShapeMap")]
    pub type JavaScriptShapeMap;

    #[wasm_bindgen(typescript_type = "JavaScriptValue")]
    pub type JavaScriptValue;

    #[wasm_bindgen(method, structural, getter)]
    pub fn tag(this: &JavaScriptValue) -> String;

    #[wasm_bindgen(method, structural, getter)]
    pub fn val(tagthis: &JavaScriptValue) -> JsValue;

    #[wasm_bindgen(typescript_type = "JavaScriptModuleDefinition")]
    pub type JavaScriptModuleDefinition;

    #[wasm_bindgen(method, structural, getter)]
    pub fn inputs(this: &JavaScriptModuleDefinition) -> JsValue;

    #[wasm_bindgen(method, structural, getter)]
    pub fn outputs(this: &JavaScriptModuleDefinition) -> JsValue;

    #[wasm_bindgen(method, structural, getter)]
    pub fn body(this: &JavaScriptModuleDefinition) -> String;
}

impl From<JavaScriptValueMap> for IoValues {
    fn from(values: JavaScriptValueMap) -> Self {
        let values = Object::from(JsValue::from(values));
        let input_keys = Object::keys(&values);
        let mut result = BTreeMap::new();

        for key in input_keys {
            let (Some(key), Some(value)) = (key.as_string(), Reflect::get(&values, &key).ok())
            else {
                continue;
            };

            let value = Value::new(value);
            let Some(runtime_value) = RuntimeValue::try_from(value).ok() else {
                continue;
            };

            result.insert(key.clone(), runtime_value);
        }

        IoValues::from(result)
    }
}

impl From<IoValues> for JavaScriptValueMap {
    fn from(value: IoValues) -> Self {
        let result = Object::new();

        for (key, value) in value.into_inner() {
            if let Err(error) = Reflect::set(
                &result,
                &JsValue::from(key),
                &Object::from(JsValue::from(Value::from(value))),
            ) {
                warn!("Failed to copy value over to JS: {:?}", error);
            }
        }

        result.unchecked_into()
    }
}
