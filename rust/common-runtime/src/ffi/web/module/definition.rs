use std::collections::BTreeMap;

use common_wit::Target;
use web_sys::js_sys::{Object, Reflect};

use crate::{
    ffi::{
        web::{cast::to_string, host::JavaScriptModuleDefinition},
        Value,
    },
    Affinity, ContentType, IoShape, IoValues, ModuleBody, RemoteFunctionDefinition, SourceCode,
    Value as RuntimeValue, ValueKind,
};

/// An enumeration of all the supported [`ModuleDefinition`]s
pub enum ModuleDefinition {
    /// A remote function definition
    RemoteFunction(RemoteFunctionDefinition),
}

impl ModuleDefinition {
    /// Parse a JavaScript-sent module definition; this is a duck-typed value
    /// that cannot be automatically converted to the strictly typed Rust
    /// counterpart.
    pub fn interpret_host_definition(
        definition: JavaScriptModuleDefinition,
    ) -> Result<(Self, IoValues), String> {
        debug!("START");
        let body = ModuleBody::SourceCode(BTreeMap::from([(
            "module.js".to_owned(),
            SourceCode {
                content_type: ContentType::JavaScript,
                body: definition.body().into(),
            },
        )]));

        debug!("A");

        let inputs = Object::from(definition.inputs());
        let outputs = Object::from(definition.outputs());

        debug!("B");
        let mut default_inputs = BTreeMap::new();
        let mut input_shape = BTreeMap::new();

        let input_keys = Object::keys(&inputs);

        debug!("C");
        for key in input_keys {
            let (Some(key), Some(value)) = (key.as_string(), Reflect::get(&inputs, &key).ok())
            else {
                continue;
            };

            let value = Value::new(value);
            let runtime_value = RuntimeValue::try_from(value)?;
            let value_kind = ValueKind::try_from(&runtime_value).map_err(to_string)?;

            default_inputs.insert(key.clone(), runtime_value);
            input_shape.insert(key, value_kind);
        }

        let mut output_shape = BTreeMap::new();

        let output_keys = Object::keys(&outputs);

        for key in output_keys {
            let (Some(key), Some(Some(value))) = (
                key.as_string(),
                Reflect::get(&outputs, &key)
                    .ok()
                    .map(|some| some.as_string()),
            ) else {
                continue;
            };

            let value_kind = ValueKind::try_from(value.as_str()).map_err(to_string)?;

            output_shape.insert(key, value_kind);
        }

        Ok((
            ModuleDefinition::RemoteFunction(
                crate::ModuleDefinition {
                    target: Target::CommonFunctionVm,
                    affinity: Affinity::RemoteOnly,
                    inputs: IoShape::from(input_shape),
                    outputs: IoShape::from(output_shape),
                    body,
                }
                .try_into()
                .map_err(to_string)?,
            ),
            IoValues::from(default_inputs),
        ))
    }
}
