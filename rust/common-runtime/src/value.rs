use crate::CommonRuntimeError;
use common_protos::common as proto;

/// An intrinsic value type within a Common Runtime
#[derive(PartialEq, Clone, Debug)]
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

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(_) => ValueKind::String,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Number(_) => ValueKind::Number,
            Value::Buffer(_) => ValueKind::Buffer,
        }
    }
}

impl TryFrom<proto::Value> for Value {
    type Error = CommonRuntimeError;

    fn try_from(value: proto::Value) -> Result<Self, Self::Error> {
        let value = value.variant.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match value {
            proto::value::Variant::String(string) => Value::String(string),
            proto::value::Variant::Number(number) => Value::Number(number),
            proto::value::Variant::Boolean(boolean) => Value::Boolean(boolean),
            proto::value::Variant::Buffer(buffer) => Value::Buffer(buffer),
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

impl From<Value> for proto::Value {
    fn from(value: Value) -> Self {
        proto::Value {
            variant: Some(match value {
                Value::String(string) => proto::value::Variant::String(string),
                Value::Boolean(number) => proto::value::Variant::Boolean(number),
                Value::Number(boolean) => proto::value::Variant::Number(boolean),
                Value::Buffer(buffer) => proto::value::Variant::Buffer(buffer),
            }),
        }
    }
}

impl From<&ValueKind> for proto::ValueKind {
    fn from(value: &ValueKind) -> Self {
        match value {
            ValueKind::String => proto::ValueKind::String,
            ValueKind::Boolean => proto::ValueKind::Boolean,
            ValueKind::Number => proto::ValueKind::Number,
            ValueKind::Buffer => proto::ValueKind::Buffer,
        }
    }
}

impl From<proto::ValueKind> for ValueKind {
    fn from(value_kind: proto::ValueKind) -> Self {
        match value_kind {
            proto::ValueKind::String => ValueKind::String,
            proto::ValueKind::Boolean => ValueKind::Boolean,
            proto::ValueKind::Number => ValueKind::Number,
            proto::ValueKind::Buffer => ValueKind::Buffer,
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        String::from(value).into()
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Buffer(value)
    }
}
