#![allow(missing_docs)]

pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

pub mod common {
    tonic::include_proto!("common");
}

pub mod builder {
    tonic::include_proto!("builder");
}

pub mod runtime {
    tonic::include_proto!("runtime");
}

use crate::{
    CommonRuntimeError, ContentType, InstantiationMode, ModuleSource, RuntimeIo, SourceCode, Value,
    ValueKind,
};
use common_wit::Target;
use std::collections::{BTreeMap, HashMap};

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

impl From<common::ModuleSource> for ModuleSource {
    fn from(value: common::ModuleSource) -> Self {
        ModuleSource {
            target: match value.target() {
                common::Target::CommonModule => Target::CommonModule,
            },
            source_code: value
                .source_code
                .into_iter()
                .map(|(name, source_code)| (name, source_code.into()))
                .collect(),
        }
    }
}

impl From<ModuleSource> for common::ModuleSource {
    fn from(value: ModuleSource) -> Self {
        common::ModuleSource {
            target: match value.target {
                Target::CommonModule => common::Target::CommonModule.into(),
            },
            source_code: value
                .source_code
                .into_iter()
                .map(|(name, source_code)| (name, source_code.into()))
                .collect(),
        }
    }
}

impl From<common::SourceCode> for SourceCode {
    fn from(value: common::SourceCode) -> Self {
        SourceCode {
            content_type: ContentType::from(value.content_type()),
            body: value.body.into(),
        }
    }
}

impl From<SourceCode> for common::SourceCode {
    fn from(value: SourceCode) -> Self {
        common::SourceCode {
            content_type: common::ContentType::from(value.content_type).into(),
            body: value.body.into(),
        }
    }
}

impl From<common::ContentType> for ContentType {
    fn from(value: common::ContentType) -> Self {
        match value {
            common::ContentType::JavaScript => ContentType::JavaScript,
            common::ContentType::Python => ContentType::Python,
        }
    }
}

impl From<ContentType> for common::ContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::JavaScript => common::ContentType::JavaScript,
            ContentType::Python => common::ContentType::Python,
        }
    }
}

impl From<runtime::InstantiationMode> for InstantiationMode {
    fn from(value: runtime::InstantiationMode) -> Self {
        match value {
            runtime::InstantiationMode::Compile => InstantiationMode::Compile,
            runtime::InstantiationMode::Interpret => InstantiationMode::Interpret,
        }
    }
}

impl From<InstantiationMode> for runtime::InstantiationMode {
    fn from(value: InstantiationMode) -> Self {
        match value {
            InstantiationMode::Compile => runtime::InstantiationMode::Compile,
            InstantiationMode::Interpret => runtime::InstantiationMode::Interpret,
        }
    }
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
