use std::fmt::Display;

use wasm_bindgen::JsValue;

pub(crate) fn js_value_as_rust_string(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{:?}", value))
}

pub(crate) fn to_string<T: Display>(value: T) -> String {
    format!("{value}")
}
