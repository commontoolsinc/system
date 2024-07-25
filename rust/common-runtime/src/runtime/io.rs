use common_protos::common;

use crate::{CommonRuntimeError, InputOutput, OutputShape, Value, ValueKind};
use std::collections::{BTreeMap, HashMap};

/// An implementation of [InputOutput] that is suitable for use with a
/// [Runtime].
#[derive(Debug, Default, Clone)]
pub struct RuntimeIo {
    input: BTreeMap<String, Value>,
    output_shape: BTreeMap<String, ValueKind>,
    output: BTreeMap<String, Value>,
}

impl RuntimeIo {
    /// Instantiate a [RuntimeIo], providing initial input state, and the
    /// expected shape of output state.
    pub fn new(input: BTreeMap<String, Value>, output_shape: BTreeMap<String, ValueKind>) -> Self {
        Self {
            input,
            output_shape,
            output: BTreeMap::new(),
        }
    }
}

impl InputOutput for RuntimeIo {
    fn read(&self, key: &str) -> Option<Value> {
        self.input.get(key).cloned()
    }

    fn write(&mut self, key: &str, value: Value) {
        if let Some(kind) = self.output_shape.get(key) {
            if value.is_of_kind(kind) {
                self.output.insert(key.into(), value);
            } else {
                warn!("Ignoring write with unexpected shape to '{key}'");
            }
        } else {
            warn!("Ignoring write to unexpected output key '{key}'");
        }
    }

    fn output(&self) -> &BTreeMap<String, Value> {
        &self.output
    }

    fn output_shape(&self) -> &OutputShape {
        &self.output_shape
    }
}

impl TryFrom<(HashMap<String, common::Value>, HashMap<String, i32>)> for RuntimeIo {
    type Error = CommonRuntimeError;

    fn try_from(
        (input_proto, output_shape_proto): (HashMap<String, common::Value>, HashMap<String, i32>),
    ) -> Result<Self, Self::Error> {
        let mut input = BTreeMap::new();
        for (key, value) in input_proto.into_iter() {
            input.insert(key, Value::try_from(value)?);
        }

        let mut output_shape = BTreeMap::new();

        for (key, value_kind) in output_shape_proto.into_iter() {
            let value_kind = common::ValueKind::try_from(value_kind)
                .map_err(|_| CommonRuntimeError::InvalidValue)?;
            output_shape.insert(key, ValueKind::from(value_kind));
        }

        Ok(RuntimeIo::new(input, output_shape))
    }
}
