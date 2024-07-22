#![allow(missing_docs)]

pub mod common_module {
    wasmtime::component::bindgen!({
      world: "common",
      path: "../../typescript/common/module/wit",
      async: true
    });
}

pub mod common_script {
    wasmtime::component::bindgen!({
      world: "common",
      path: "../../typescript/common/script/wit",
      async: true,
      with: {
          "common:module": crate::bindings::common_module::common::module,
          "common:data": crate::bindings::common_module::common::data,
          "common:io": crate::bindings::common_module::common::io,
      }
    });
}

use common_module::common::data::types::Value as BindingValue;
use common_wit::Value;

impl From<BindingValue> for Value {
    fn from(value: BindingValue) -> Self {
        match value {
            BindingValue::String(inner) => Value::String(inner),
            BindingValue::Number(inner) => Value::Number(inner),
            BindingValue::Boolean(inner) => Value::Boolean(inner),
            BindingValue::Buffer(inner) => Value::Buffer(inner),
        }
    }
}

impl From<Value> for BindingValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(inner) => BindingValue::String(inner),
            Value::Boolean(inner) => BindingValue::Boolean(inner),
            Value::Number(inner) => BindingValue::Number(inner),
            Value::Buffer(inner) => BindingValue::Buffer(inner),
        }
    }
}
