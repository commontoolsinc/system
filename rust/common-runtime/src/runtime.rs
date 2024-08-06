use crate::{
    CommonRuntimeError, InputOutput, IoData, IoShape, IoValues, ModuleDefinition, ModuleInstance,
    ModuleInstanceId, ModulePreparer, PreparedModule, ToModuleSources, ToWasmComponent, Value,
};
use common_ifc::{Context, Data, Label, ModuleEnvironment, Policy};
use std::collections::BTreeMap;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::browser::{BrowserCompiler, BrowserInterpreter};

#[cfg(not(target_arch = "wasm32"))]
use crate::wasmtime::{WasmtimeCompiler, WasmtimeInterpreter};

#[cfg(not(target_arch = "wasm32"))]
use wasmtime::{Config, Engine, OptLevel};

/// An implementation of [InputOutput] that is suitable for use with a
/// [Runtime].
#[derive(Debug, Default, Clone)]
pub struct RuntimeIo {
    input: IoData,
    output: IoData,
    output_shape: IoShape,
    label_constraints: Label,
}

impl RuntimeIo {
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

    /// Takes input values [ValueIo] and an output shape, and converts
    /// the values into [Data] with strictest labels. Used for
    /// specifying initial state.
    pub fn from_initial_state(input_values: IoValues, output_shape: IoShape) -> Self {
        let mut map = BTreeMap::new();
        for (key, value) in input_values.into_inner().into_iter() {
            map.insert(key, Data::with_strict_labels(value));
        }
        RuntimeIo::new(IoData::from(map), output_shape)
    }
}

impl InputOutput for RuntimeIo {
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

    fn output_shape(&self) -> &IoShape {
        &self.output_shape
    }
}

struct LiveModule {
    pub instance: Box<dyn ModuleInstance<InputOutput = RuntimeIo>>,
    pub output_shape: IoShape,
    pub context: Context,
}

/// A [Runtime] is the main entrypoint for all Common Module instantiation and
/// invocation. It manages the details of preparing Common Modules for
/// instantiation, and appropriately sandboxing them ahead of invocation.
pub struct Runtime {
    #[cfg(not(target_arch = "wasm32"))]
    compiler: WasmtimeCompiler<RuntimeIo>,
    #[cfg(not(target_arch = "wasm32"))]
    interpreter: WasmtimeInterpreter<RuntimeIo>,

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    compiler: BrowserCompiler<RuntimeIo>,
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    interpreter: BrowserInterpreter<RuntimeIo>,

    module_instances: BTreeMap<ModuleInstanceId, LiveModule>,

    module_environment: ModuleEnvironment,
}

impl Runtime {
    /// Initialize a new [Runtime]
    pub fn new() -> Result<Self, CommonRuntimeError> {
        #[cfg(not(target_arch = "wasm32"))]
        let wasmtime_engine = {
            let mut config = Config::default();

            config.cranelift_opt_level(OptLevel::Speed);
            config.async_support(true);
            config.wasm_backtrace(true);

            Engine::new(&config)
                .map_err(|error| CommonRuntimeError::SandboxCreationFailed(format!("{error}")))
        }?;

        #[cfg(not(target_arch = "wasm32"))]
        let module_environment = ModuleEnvironment::Server;

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        let module_environment = ModuleEnvironment::WebBrowser;

        Ok(Runtime {
            #[cfg(not(target_arch = "wasm32"))]
            compiler: WasmtimeCompiler::new(wasmtime_engine.clone())?,
            #[cfg(not(target_arch = "wasm32"))]
            interpreter: WasmtimeInterpreter::new(wasmtime_engine)?,

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            compiler: BrowserCompiler::new(),
            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            interpreter: BrowserInterpreter::new(),

            module_instances: Default::default(),
            module_environment,
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
        debug!(?output_shape);
        let prepared_module = self.compiler.prepare(module).await?;
        debug!("Retrieved prepared module");
        let instance = prepared_module.instantiate(io).await?;
        let instance_id = instance.id().clone();
        debug!(?instance_id, "Instantiated the module");

        self.module_instances.insert(
            instance_id.clone(),
            LiveModule {
                instance: Box::new(instance),
                output_shape,
                context: (self.module_environment.clone(),).into(),
            },
        );

        Ok(instance_id)
    }

    /// Instantiate the given Common Module in interpreted mode (if supported)
    #[instrument(skip(self, module, io))]
    pub async fn interpret<
        Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    >(
        &mut self,
        module: Module,
        io: RuntimeIo,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let output_shape = io.output_shape().clone();
        debug!(?output_shape);
        let prepared_module = self.interpreter.prepare(module).await?;
        debug!("Retrieved prepared module");
        let instance = prepared_module.instantiate(io).await?;
        let instance_id = instance.id().clone();
        debug!(?instance_id, "Instantiated the module");

        self.module_instances.insert(
            instance_id.clone(),
            LiveModule {
                instance: Box::new(instance),
                output_shape,
                context: (self.module_environment.clone(),).into(),
            },
        );

        Ok(instance_id)
    }

    /// For a given live Common Module instance, get the [OutputShape] that was
    /// configured at instantiation time.
    pub fn output_shape(
        &self,
        instance_id: &ModuleInstanceId,
    ) -> Result<&IoShape, CommonRuntimeError> {
        if let Some(cached) = self.module_instances.get(instance_id) {
            Ok(&cached.output_shape)
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
        policy: &Policy,
    ) -> Result<RuntimeIo, CommonRuntimeError> {
        let Some(cached) = self.module_instances.get(instance_id) else {
            return Err(CommonRuntimeError::ModuleRunFailed(format!(
                "No instance found for ID '{}'",
                instance_id
            )));
        };

        policy.validate(io.input().iter(), &cached.context)?;
        cached.instance.run(io).await
    }
}
