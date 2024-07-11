use crate::{
    wasmtime::WasmtimeCompiler, CommonRuntimeError, InputOutput, ModuleDefinition, ModuleInstance,
    ModuleInstanceId, ModulePreparer, OutputShape, PreparedModule, ToModuleSources,
    ToWasmComponent, Value, ValueKind,
};
use common_protos::common;
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

/// A [Runtime] is the main entrypoint for all Common Module instantiation and
/// invocation. It manages the details of preparing Common Modules for
/// instantiation, and appropriately sandboxing them ahead of invocation.
pub struct Runtime {
    compiler: WasmtimeCompiler<RuntimeIo>,
    module_instances: BTreeMap<
        ModuleInstanceId,
        (
            Box<dyn ModuleInstance<InputOutput = RuntimeIo>>,
            OutputShape,
        ),
    >,
}

impl Runtime {
    /// Initialize a new [Runtime]
    pub fn new() -> Result<Self, CommonRuntimeError> {
        Ok(Runtime {
            compiler: WasmtimeCompiler::new()?,
            module_instances: Default::default(),
        })
    }

    /// Instantiate the given Common Module in compiled mode (if supported)
    #[instrument(skip(self, module, io))]
    pub async fn compile<Module: ModuleDefinition + ToWasmComponent + 'static>(
        &mut self,
        module: Module,
        io: RuntimeIo,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let output_shape = io.output_shape().clone();
        debug!(?output_shape, "Collected output shape");
        let prepared_module = self.compiler.prepare(module).await?;
        debug!("Retrieved prepared module");
        let instance = prepared_module.instantiate(io).await?;
        let instance_id = instance.id().clone();
        debug!(?instance_id, "Instantiated the module");

        self.module_instances
            .insert(instance_id.clone(), (Box::new(instance), output_shape));

        Ok(instance_id)
    }

    /// Instantiate the given Common Module in interpreted mode (if supported)
    pub async fn interpret<Module: ModuleDefinition + ToModuleSources + 'static>(
        &mut self,
        _module: Module,
        _io: RuntimeIo,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        todo!();
    }

    /// For a given live Common Module instance, get the [OutputShape] that was
    /// configured at instantiation time.
    pub fn output_shape(
        &self,
        instance_id: &ModuleInstanceId,
    ) -> Result<&OutputShape, CommonRuntimeError> {
        if let Some((_, output_shape)) = self.module_instances.get(instance_id) {
            Ok(output_shape)
        } else {
            Err(CommonRuntimeError::UnknownInstanceId(instance_id.clone()))
        }
    }

    /// Invoke the interior `run` function of the Common Module instance given
    /// its instance ID
    pub async fn run(
        &self,
        instance_id: &ModuleInstanceId,
        io: RuntimeIo,
    ) -> Result<RuntimeIo, CommonRuntimeError> {
        let Some((instance, _)) = self.module_instances.get(instance_id) else {
            return Err(CommonRuntimeError::ModuleRunFailed(format!(
                "No instance found for ID '{}'",
                instance_id
            )));
        };

        instance.run(io).await
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
