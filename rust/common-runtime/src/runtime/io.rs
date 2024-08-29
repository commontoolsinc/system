use common_ifc::{Data, Label};
use std::collections::BTreeMap;

use crate::{sync::ConditionalSync, CommonRuntimeError, Value, ValueKind};
use common_macros::NewType;
use common_protos::common;
use std::collections::HashMap;

/// A wrapper type around the mapping of IO names
/// for Common Modules.
#[derive(NewType, Default, Clone, Debug)]
pub struct IoData(BTreeMap<String, Data<Value>>);

impl TryFrom<HashMap<String, common::LabeledData>> for IoData {
    type Error = CommonRuntimeError;

    fn try_from(proto: HashMap<String, common::LabeledData>) -> Result<Self, Self::Error> {
        let mut map = BTreeMap::new();
        for (key, data) in proto.into_iter() {
            map.insert(key, Data::try_from(data)?);
        }
        Ok(IoData(map))
    }
}

impl From<&IoData> for HashMap<String, common::LabeledData> {
    fn from(value: &IoData) -> Self {
        value.clone().into()
    }
}

impl From<IoData> for HashMap<String, common::LabeledData> {
    fn from(value: IoData) -> Self {
        value
            .into_inner()
            .into_iter()
            .map(|(key, data)| (key, data.into()))
            .collect()
    }
}

impl From<&IoData> for HashMap<String, common::Value> {
    fn from(value: &IoData) -> Self {
        value
            .inner()
            .iter()
            .map(|(key, data)| (key.to_owned(), data.value.clone().into()))
            .collect()
    }
}

/// A wrapper type for the mapping of IO names to value type
/// for Common Modules.
#[derive(NewType, Default, Clone, Debug, Eq, PartialEq)]
pub struct IoShape(BTreeMap<String, ValueKind>);

impl TryFrom<HashMap<String, i32>> for IoShape {
    type Error = CommonRuntimeError;
    fn try_from(value: HashMap<String, i32>) -> Result<Self, Self::Error> {
        let mut shape = BTreeMap::new();

        for (key, value_kind) in value.into_iter() {
            let value_kind = common::ValueKind::try_from(value_kind)
                .map_err(|_| CommonRuntimeError::InvalidValue)?;
            shape.insert(key, ValueKind::from(value_kind));
        }

        Ok(Self(shape))
    }
}

impl From<&IoShape> for HashMap<String, i32> {
    fn from(value: &IoShape) -> Self {
        value
            .inner()
            .iter()
            .map(|(key, value_kind)| {
                (
                    key.to_owned(),
                    common_protos::common::ValueKind::from(value_kind).into(),
                )
            })
            .collect()
    }
}

impl From<&IoData> for IoValues {
    fn from(value: &IoData) -> Self {
        IoValues(
            value
                .iter()
                .map(|(key, value)| (key.to_owned(), value.value.clone()))
                .collect(),
        )
    }
}

impl From<&IoData> for IoShape {
    fn from(value: &IoData) -> Self {
        IoShape(
            value
                .iter()
                .map(|(key, value)| (key.to_owned(), ValueKind::from(&value.value)))
                .collect(),
        )
    }
}

impl From<&IoValues> for IoShape {
    fn from(value: &IoValues) -> Self {
        IoShape(
            value
                .iter()
                .map(|(key, value)| (key.to_owned(), ValueKind::from(value)))
                .collect(),
        )
    }
}

/// A wrapper type for the mapping of IO names to default values
/// without labels for Common Modules.
#[derive(NewType, Default, Clone, Debug)]
pub struct IoValues(BTreeMap<String, Value>);

impl TryFrom<HashMap<String, common::Value>> for IoValues {
    type Error = CommonRuntimeError;
    fn try_from(proto: HashMap<String, common::Value>) -> Result<Self, Self::Error> {
        let mut map = BTreeMap::new();
        for (key, value) in proto.into_iter() {
            map.insert(key, Value::try_from(value)?);
        }
        Ok(Self(map))
    }
}

/// A generic trait for a reference to state. The implementation may embody
/// state that is opaque, readable and/or writable.
pub trait InputOutput: Clone + Default + ConditionalSync + std::fmt::Debug {
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

    /// Get a mapping of the input keys to their set [Data].
    fn input(&self) -> &IoData;

    /// Get a mapping of the output keys to their set [Data]. Keys with no set
    /// values will not be pressent in the output, even if they were allowed to
    /// be set.
    fn output(&self) -> &IoData;

    /// Get a mutable reference to the output [Data]
    fn output_mut(&mut self) -> &mut IoData;

    /// Get the shape of the output, which is the expected [ValueKind] that maps
    /// to each allowed key in the output space
    fn output_shape(&self) -> &IoShape;
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

    fn input(&self) -> &IoData {
        self.as_ref().input()
    }

    fn output(&self) -> &IoData {
        self.as_ref().output()
    }

    fn output_mut(&mut self) -> &mut IoData {
        self.as_mut().output_mut()
    }

    fn output_shape(&self) -> &IoShape {
        self.as_ref().output_shape()
    }
}

/// An implementation of [InputOutput] that is suitable for use with many kinds
/// of [crate::ModuleDriver].
#[derive(Debug, Default, Clone)]
pub struct BasicIo {
    input: IoData,
    output: IoData,
    output_shape: IoShape,
    label_constraints: Label,
}

impl BasicIo {
    /// Instantiate a [RuntimeIo], providing initial input state, and the
    /// expected shape of output state.
    pub fn new(input: IoData, output_shape: IoShape) -> Self {
        let label_constraints = Label::constrain(input.iter());
        Self {
            input,
            output_shape,
            output: IoData::default(),
            label_constraints,
        }
    }

    /// Takes input values [IoValues] and an output shape [IoShape], and converts
    /// the values into [Data] with strictest labels. Used for
    /// specifying initial state.
    pub fn from_initial_state(input_values: IoValues, output_shape: IoShape) -> Self {
        let mut map = BTreeMap::new();
        for (key, value) in input_values.into_inner().into_iter() {
            map.insert(key, Data::with_strict_labels(value));
        }
        BasicIo::new(IoData::from(map), output_shape)
    }
}

impl InputOutput for BasicIo {
    fn read(&self, key: &str) -> Option<Value> {
        self.input.get(key).map(|d| d.value.clone())
    }

    fn write(&mut self, key: &str, value: Value) {
        if let Some(kind) = self.output_shape.get(key) {
            if value.is_of_kind(kind) {
                let data = Data::from((value, self.label_constraints.clone()));
                self.output.insert(key.into(), data);
            } else {
                warn!("Ignoring write with unexpected shape to '{key}'");
            }
        } else {
            warn!("Ignoring write to unexpected output key '{key}'");
        }
    }

    fn input(&self) -> &IoData {
        &self.input
    }

    fn output(&self) -> &IoData {
        &self.output
    }

    fn output_mut(&mut self) -> &mut IoData {
        &mut self.output
    }

    fn output_shape(&self) -> &IoShape {
        &self.output_shape
    }
}
