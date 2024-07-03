use crate::{
    protos::{self, common::value::ValueType},
    CommonRuntimeError,
};

/// An intrinsic value type within a Common Runtime
#[derive(Clone, Debug)]
pub enum Value {
    /// A UTF-8 string
    String(String),
    /// A boolean: true or false
    Boolean(bool),
    /// A double-precision floating-point number
    Number(f64),
    /// A slab of bytes
    Buffer(Vec<u8>),
}

impl TryFrom<protos::common::Value> for Value {
    type Error = CommonRuntimeError;

    fn try_from(value: protos::common::Value) -> Result<Self, Self::Error> {
        let value = value.value_type.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match value {
            ValueType::String(string) => Value::String(string),
            ValueType::Number(number) => Value::Number(number),
            ValueType::Boolean(boolean) => Value::Boolean(boolean),
            ValueType::Buffer(buffer) => Value::Buffer(buffer),
        })
    }
}

impl From<Value> for protos::common::Value {
    fn from(value: Value) -> Self {
        protos::common::Value {
            value_type: Some(match value {
                Value::String(string) => ValueType::String(string),
                Value::Boolean(number) => ValueType::Boolean(number),
                Value::Number(boolean) => ValueType::Number(boolean),
                Value::Buffer(buffer) => ValueType::Buffer(buffer),
            }),
        }
    }
}
