use crate::{
    CommonRuntimeError, InputOutput, ModuleDefinition, ModuleInstance, ModuleInstanceId,
    ModulePreparer, OutputShape, PreparedModule, Schedule, ToModuleSources, ToWasmComponent, Value,
    ValueKind,
};
use common_protos::common;
use http::Uri;
use std::collections::{BTreeMap, HashMap};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::browser::{BrowserCompiler, BrowserInterpreter};

#[cfg(not(target_arch = "wasm32"))]
use crate::wasmtime::{WasmtimeCompiler, WasmtimeInterpreter};

#[cfg(not(target_arch = "wasm32"))]
use wasmtime::{Config, Engine, OptLevel};

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

    module_instances: BTreeMap<
        ModuleInstanceId,
        (
            Box<dyn ModuleInstance<InputOutput = RuntimeIo>>,
            OutputShape,
        ),
    >,
    // remote_runtime_address: Option<Uri>,
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
        })
    }

    /// Instantiate the given Common Module in compiled mode (if supported)
    #[instrument(skip(self, module, io))]
    pub async fn compile<Module: ModuleDefinition + ToWasmComponent + 'static>(
        &mut self,
        module: Module,
        io: RuntimeIo,
        schedule: Schedule,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let output_shape = io.output_shape().clone();
        debug!(?output_shape);
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
    #[instrument(skip(self, module, io))]
    pub async fn interpret<
        Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    >(
        &mut self,
        module: Module,
        io: RuntimeIo,
        schedule: Schedule,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let output_shape = io.output_shape().clone();
        debug!(?output_shape);
        let prepared_module = self.interpreter.prepare(module).await?;
        debug!("Retrieved prepared module");
        let instance = prepared_module.instantiate(io).await?;
        let instance_id = instance.id().clone();
        debug!(?instance_id, "Instantiated the module");

        self.module_instances
            .insert(instance_id.clone(), (Box::new(instance), output_shape));

        Ok(instance_id)
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
