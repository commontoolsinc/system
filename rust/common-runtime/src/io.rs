use std::collections::BTreeMap;

use crate::{sync::ConditionalSync, Value};

/// A generic trait for a reference to state. The implementation may embody
/// state that is opaque, readable and/or writable.
pub trait InputOutput: ConditionalSync + std::fmt::Debug {
    /// Attempt to read some [Value] from state that is assigned some well-known
    /// `key`. A value may be returned if it is part of the state, and the reader
    /// is allowed to read it.
    fn read(&self, key: &str) -> Option<Value>;

    /// Write some [Value] to a well-known `key`. The write may or may not be
    /// accepted. There is no prescription made as to the transactional
    /// guarantees of a call to `write`. Subsequent calls to `read` for the same
    /// `key` may or may not reflect the effect of a `write`, regardless of
    /// whether or not it was considered to be successful.
    fn write(&mut self, key: &str, value: Value);

    /// Get a mapping of the output keys to their set values. Keys with no set
    /// values will not be pressent in the output, even if they were allowed to
    /// be set.
    fn output(&self) -> &BTreeMap<String, Value>;
}

impl<Io> InputOutput for Box<Io>
where
    Io: InputOutput,
{
    fn read(&self, key: &str) -> Option<Value> {
        self.as_ref().read(key)
    }

    fn write(&mut self, key: &str, value: Value) {
        self.as_mut().write(key, value)
    }

    fn output(&self) -> &BTreeMap<String, Value> {
        self.as_ref().output()
    }
}
