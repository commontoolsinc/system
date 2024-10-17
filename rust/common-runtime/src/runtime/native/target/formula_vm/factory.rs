use crate::{
    module::FormulaVmDefinition,
    target::formula_vm::{NativeFormulaVm, NativeFormulaVmContext, VirtualModule},
    ArtifactResolver, CommonRuntimeError, ContentType, ModuleFactory, VirtualModuleInterpreter,
};
use async_trait::async_trait;
use std::sync::Arc;
use wasmtime::{
    component::{Component, Linker},
    Engine as WasmtimeEngine, Store,
};

/// An implementor of [ModuleFactory] for [NativeFormulaVm] Modules that may be
/// instantiated by a [crate::NativeRuntime]
#[derive(Clone)]
pub struct NativeFormulaVmFactory {
    engine: WasmtimeEngine,
    definition: Arc<FormulaVmDefinition>,
    linker: Linker<NativeFormulaVmContext>,
    interpreter: Arc<Component>,
    source_code: Arc<String>,
}

impl NativeFormulaVmFactory {
    /// Given a [crate::ModuleDefinition], select an appropriate
    /// [VirtualModuleInterpreter] to be used as the VM to host the Module's
    /// source code
    pub fn select_virtual_module_interpreter(
        definition: &FormulaVmDefinition,
    ) -> Result<VirtualModuleInterpreter, CommonRuntimeError> {
        Ok(match definition.content_type()? {
            ContentType::JavaScript => VirtualModuleInterpreter::JavaScriptFormula,
            any_other => {
                return Err(CommonRuntimeError::PreparationFailed(format!(
                    "{any_other} is not a supported language for a virtual module"
                )))
            }
        })
    }

    /// Given a [WasmtimeEngine], a [VirtualModuleInterpreter] and an
    /// [ArtifactResolver], perform the steps necessary to prepare a
    /// corresponding VM
    pub async fn prepare_interpreter(
        engine: WasmtimeEngine,
        virtual_module_interpreter: VirtualModuleInterpreter,
        artifact_resolver: ArtifactResolver,
    ) -> Result<Component, CommonRuntimeError> {
        let wasm_bytes = artifact_resolver
            .get_virtual_module_interpreter_wasm(virtual_module_interpreter)
            .await?;

        let component = Component::new(&engine, wasm_bytes)
            .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?;

        Ok(component)
    }

    /// Instantiate a new [NativeFormulaVmFactory] for a given
    /// [crate::ModuleDefinition] and various Wasm runtime acoutrement
    pub async fn new(
        engine: WasmtimeEngine,
        artifact_resolver: ArtifactResolver,
        interpreter: Arc<Component>,
        definition: FormulaVmDefinition,
    ) -> Result<Self, CommonRuntimeError> {
        let source_code = artifact_resolver
            .get_bundled_source_code(&definition)
            .await?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        //bindings::add_to_linker(&mut linker)
        //    .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        Ok(NativeFormulaVmFactory {
            engine,
            definition: Arc::new(definition),
            linker,
            interpreter,
            source_code,
        })
    }
}

#[async_trait]
impl ModuleFactory for NativeFormulaVmFactory {
    type Context = NativeFormulaVmContext;

    type Module = NativeFormulaVm;

    async fn instantiate(
        &self,
        context: Self::Context,
    ) -> Result<Self::Module, CommonRuntimeError> {
        let mut store = Store::new(&self.engine, context);

        let virtual_module =
            VirtualModule::instantiate_async(&mut store, &self.interpreter, &self.linker)
                .await
                .map_err(|error| {
                    CommonRuntimeError::ModuleInstantiationFailed(format!("{error}"))
                })?;

        virtual_module
            .call_set_source(&mut store, &self.source_code)
            .await
            .map_err(|error| CommonRuntimeError::ModuleInstantiationFailed(format!("{error}")))?
            .map_err(|error| {
                CommonRuntimeError::ModuleInstantiationFailed(format!("Script error: {error}"))
            })?;

        NativeFormulaVm::new(self.definition.clone(), store, virtual_module)
    }
}
