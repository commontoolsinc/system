use crate::Result;
use js_sys::{Uint8Array, JSON};
use wasm_bindgen::JsValue;

pub fn js_to_string(value: JsValue) -> Result<String> {
    Ok(value
        .as_string()
        .ok_or_else(|| String::from("Expected value to be string."))?)
}

pub fn serialize_js(value: &JsValue) -> Result<String> {
    match JSON::stringify(value) {
        Ok(js_string) => Ok(js_to_string(JsValue::from(js_string))?),
        Err(js_string) => Err(js_to_string(js_string)?.into()),
    }
}

pub fn deserialize_js(value: &str) -> Result<JsValue> {
    match JSON::parse(value) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(js_to_string(e)?.into()),
    }
}

pub fn bytes_to_typed_array(bytes: &[u8]) -> Result<JsValue> {
    Ok(JsValue::from(Uint8Array::from(bytes)))
}
