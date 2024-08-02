use crate::CommonRuntimeError;
use common_protos::common;

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

impl Value {
    /// Check if a [ValueKind] corresponds to the type of this [Value]
    pub fn is_of_kind(&self, kind: &ValueKind) -> bool {
        match self {
            Value::String(_) if kind == &ValueKind::String => true,
            Value::Boolean(_) if kind == &ValueKind::Boolean => true,
            Value::Number(_) if kind == &ValueKind::Number => true,
            Value::Buffer(_) if kind == &ValueKind::Buffer => true,
            _ => false,
        }
    }
}

/// The set of variant types for [Value]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ValueKind {
    /// A UTF-8 string
    String,
    /// A boolean: true or false
    Boolean,
    /// A double-precision floating-point number
    Number,
    /// A slab of bytes
    Buffer,
}

impl TryFrom<common::Value> for Value {
    type Error = CommonRuntimeError;

    fn try_from(value: common::Value) -> Result<Self, Self::Error> {
        let value = value.variant.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match value {
            common::value::Variant::String(string) => Value::String(string),
            common::value::Variant::Number(number) => Value::Number(number),
            common::value::Variant::Boolean(boolean) => Value::Boolean(boolean),
            common::value::Variant::Buffer(buffer) => Value::Buffer(buffer),
        })
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::String(v) => v.to_string(),
                Value::Boolean(v) => v.to_string(),
                Value::Number(v) => v.to_string(),
                Value::Buffer(v) => format!("Buffer<{}b>", v.len()),
            }
        )
    }
}

impl From<Value> for common::Value {
    fn from(value: Value) -> Self {
        common::Value {
            variant: Some(match value {
                Value::String(string) => common::value::Variant::String(string),
                Value::Boolean(number) => common::value::Variant::Boolean(number),
                Value::Number(boolean) => common::value::Variant::Number(boolean),
                Value::Buffer(buffer) => common::value::Variant::Buffer(buffer),
            }),
        }
    }
}

impl From<common::ValueKind> for ValueKind {
    fn from(value_kind: common::ValueKind) -> Self {
        match value_kind {
            common::ValueKind::String => ValueKind::String,
            common::ValueKind::Boolean => ValueKind::Boolean,
            common::ValueKind::Number => ValueKind::Number,
            common::ValueKind::Buffer => ValueKind::Buffer,
        }
    }
}
