use crate::Value as RuntimeValue;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{JsString, Number, Reflect, Uint8Array};

/// An intermediate representation of a Runtime-legible value, used
/// as fulcrum for transformation between plain JavaScript objects and
/// strictly typed Rust [crate::Value].
#[wasm_bindgen(getter_with_clone)]
pub struct Value {
    /// A string that represents the underlying type of the value
    pub tag: String,
    /// The raw [JsValue] representation of the value
    pub val: JsValue,
}

#[wasm_bindgen]
impl Value {
    /// Construct a new [`Value`] from a raw JavaScript value. A plain,
    /// un-tagged JavaScript value will be inferred, so this can be constructed
    /// with "just a string" or "just a number" or "just a Uint8Array" etc.
    #[wasm_bindgen(constructor)]
    pub fn new(inner: JsValue) -> Self {
        match inner.js_typeof().as_string().unwrap_or_default().as_str() {
            "string" => {
                return Self {
                    tag: "string".into(),
                    val: inner,
                };
            }
            "boolean" => {
                return Self {
                    tag: "boolean".into(),
                    val: inner,
                };
            }
            "number" if !Number::is_nan(&inner) => {
                return Self {
                    tag: "number".into(),
                    val: inner,
                };
            }
            "object" => {
                if inner.is_instance_of::<Uint8Array>() {
                    return Self {
                        tag: "buffer".into(),
                        val: inner,
                    };
                }

                let tag = Reflect::get(&inner, &JsValue::from(JsString::from("tag"))).ok();
                let val = Reflect::get(&inner, &JsValue::from(JsString::from("val"))).ok();

                if let (Some(tag), Some(val)) = (tag, val) {
                    if let (Some(tag), false) = (tag.try_into().ok(), val.is_undefined()) {
                        return Self { tag, val };
                    };
                };
            }
            _ => (),
        };

        warn!(
            "Could not interpret '{:?}' as a supported value kind",
            inner
        );

        return Self {
            tag: "boolean".into(),
            val: JsValue::from_bool(false),
        };
    }
}

impl From<RuntimeValue> for Value {
    fn from(value: RuntimeValue) -> Self {
        match value {
            RuntimeValue::String(string) => Value {
                tag: "string".into(),
                val: string.into(),
            },
            RuntimeValue::Boolean(boolean) => Value {
                tag: "boolean".into(),
                val: boolean.into(),
            },
            RuntimeValue::Number(number) => Value {
                tag: "number".into(),
                val: number.into(),
            },
            RuntimeValue::Buffer(buffer) => Value {
                tag: "buffer".into(),
                val: buffer.into(),
            },
        }
    }
}

impl TryFrom<Value> for RuntimeValue {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value.tag.as_str() {
            "string" => {
                if let Some(string) = value.val.as_string() {
                    crate::Value::String(string)
                } else {
                    return Err(format!("Expected a string value, got {:?}", value.val));
                }
            }
            "number" => {
                if let Some(number) = value.val.as_f64() {
                    crate::Value::Number(number)
                } else {
                    return Err(format!("Expected a number value, got {:?}", value.val));
                }
            }
            "boolean" => {
                if let Some(boolean) = value.val.as_bool() {
                    crate::Value::Boolean(boolean)
                } else {
                    return Err(format!("Expected a boolean value, got {:?}", value.val));
                }
            }
            "buffer" => {
                let array = JsValue::dyn_into::<Uint8Array>(value.val).map_err(|val| {
                    format!(
                        "Expected buffer (Uint8Array), but could not cast given value {:?}",
                        val
                    )
                })?;
                crate::Value::Buffer(array.to_vec())
            }
            _ => return Err(format!("Unrecognized value kind '{}'", value.tag)),
        })
    }
}
