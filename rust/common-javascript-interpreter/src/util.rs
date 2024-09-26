use boa_engine::{JsError, JsValue};

/// Create a [JsError] from a string.
pub fn js_error<S: Into<String>>(message: S) -> JsError {
    JsError::from_opaque(JsValue::String(message.into().into()))
}
