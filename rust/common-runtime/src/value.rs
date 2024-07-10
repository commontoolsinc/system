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
