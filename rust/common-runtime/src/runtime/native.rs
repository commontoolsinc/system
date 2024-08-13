use std::sync::Arc;

use async_trait::async_trait;
use target::{function::NativeFunctionFactory, function_vm::NativeFunctionVmFactory};
use wasmtime::{
    component::Component, Config as WasmtimeConfig, Engine as WasmtimeEngine, OptLevel,
};

use crate::{
    CommonRuntimeError,
    {
        cache::Cache,
        module::{FunctionDefinition, FunctionVmDefinition, ModuleId},
        ModuleDriver,
    },
};

mod artifact;
pub use artifact::*;

pub mod target;

/// A Runtime implementation for "native" contexts, where "native" is a loose
/// reference to the native architecture of the local machine, and is used to
/// distinguish the Runtime from one that may run in a virtual machine or web
/// browser.
///
/// The [NativeRuntime] implements Module types using Wasmtime as its internal
/// Wasm runtime.
pub struct NativeRuntime {
    artifact_resolver: ArtifactResolver,
    wasmtime_engine: WasmtimeEngine,

    function_cache: Cache<ModuleId, NativeFunctionFactory>,
    function_vm_cache: Cache<ModuleId, NativeFunctionVmFactory>,

    vm_interpreter_cache: Cache<VirtualModuleInterpreter, Arc<Component>>,
}

impl NativeRuntime {
    /// Instantiate the [NativeRuntime] using an [ArtifactResolver]
    pub fn new(artifact_resolver: ArtifactResolver) -> Result<Self, CommonRuntimeError> {
        let wasmtime_engine = {
            let mut config = WasmtimeConfig::default();

            config.cranelift_opt_level(OptLevel::Speed);
            config.async_support(true);
            config.wasm_backtrace(true);

            WasmtimeEngine::new(&config)
                .map_err(|error| CommonRuntimeError::SandboxCreationFailed(format!("{error}")))
        }?;

        Ok(NativeRuntime {
            artifact_resolver,
            wasmtime_engine,

            function_cache: Cache::new(32)?,
            function_vm_cache: Cache::new(32)?,

            vm_interpreter_cache: Cache::new(16)?,
        })
    }
}

#[async_trait]
impl ModuleDriver<FunctionDefinition> for NativeRuntime {
    type ModuleFactory = NativeFunctionFactory;

    async fn prepare(
        &self,
        definition: FunctionDefinition,
    ) -> Result<Self::ModuleFactory, CommonRuntimeError> {
        let id = ModuleId::from(&*definition);
        let factory = if let Some(cached_item) = self.function_cache.get(&id).await {
            cached_item
        } else {
            let factory = NativeFunctionFactory::new(
                self.wasmtime_engine.clone(),
                self.artifact_resolver.clone(),
                definition,
            )
            .await?;
            self.function_cache.insert(id, factory.clone()).await;
            factory
        };
        Ok(factory)
    }
}

#[async_trait]
impl ModuleDriver<FunctionVmDefinition> for NativeRuntime {
    type ModuleFactory = NativeFunctionVmFactory;

    async fn prepare(
        &self,
        definition: FunctionVmDefinition,
    ) -> Result<Self::ModuleFactory, CommonRuntimeError> {
        let id = ModuleId::from(&*definition);

        let factory = if let Some(cached_item) = self.function_vm_cache.get(&id).await {
            cached_item
        } else {
            let virtual_module_interpreter =
                NativeFunctionVmFactory::select_virtual_module_interpreter(&definition)?;

            let interpreter = if let Some(cached_item) = self
                .vm_interpreter_cache
                .get(&virtual_module_interpreter)
                .await
            {
                cached_item
            } else {
                let interpreter = Arc::new(
                    NativeFunctionVmFactory::prepare_interpreter(
                        self.wasmtime_engine.clone(),
                        virtual_module_interpreter.clone(),
                        self.artifact_resolver.clone(),
                    )
                    .await?,
                );
                self.vm_interpreter_cache
                    .insert(virtual_module_interpreter, interpreter.clone())
                    .await;
                interpreter
            };

            let factory = NativeFunctionVmFactory::new(
                self.wasmtime_engine.clone(),
                self.artifact_resolver.clone(),
                interpreter,
                definition,
            )
            .await?;
            self.function_vm_cache.insert(id, factory.clone()).await;
            factory
        };
        Ok(factory)
    }
}
