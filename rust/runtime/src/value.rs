/// An intrinsic value type within a Common Runtime
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
