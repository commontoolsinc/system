use crate::{
    target::{
        bindings::module::{add_to_linker, Module},
        function::{NativeFunction, NativeFunctionContext},
    },
    CommonRuntimeError,
    {module::FunctionDefinition, ArtifactResolver, ModuleFactory},
};
use async_trait::async_trait;
use std::sync::Arc;
use wasmtime::{
    component::{Component, Linker},
    Engine as WasmtimeEngine, Store,
};

/// An implementor of [ModuleFactory] for [NativeFunction] Modules that may be
/// instantiated by a [crate::NativeRuntime]
#[derive(Clone)]
pub struct NativeFunctionFactory {
    engine: WasmtimeEngine,
    definition: Arc<FunctionDefinition>,
    linker: Linker<NativeFunctionContext>,
    component: Component,
}

impl NativeFunctionFactory {
    /// Instantiate a new [NativeFunctionFactory] for a given
    /// [crate::ModuleDefinition] and various Wasm runtime acoutrement
    pub async fn new(
        engine: WasmtimeEngine,
        artifact_resolver: ArtifactResolver,
        definition: FunctionDefinition,
    ) -> Result<Self, CommonRuntimeError> {
        let wasm_bytes = artifact_resolver.get_module_wasm(&definition).await?;

        let component = Component::new(&engine, wasm_bytes)
            .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        wasmtime_wasi_http::proxy::add_only_http_to_linker(&mut linker)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        add_to_linker(&mut linker)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        Ok(NativeFunctionFactory {
            engine,
            definition: Arc::new(definition),
            linker,
            component,
        })
    }
}

#[async_trait]
impl ModuleFactory for NativeFunctionFactory {
    type Context = NativeFunctionContext;

    type Module = NativeFunction;

    async fn instantiate(
        &self,
        context: Self::Context,
    ) -> Result<Self::Module, CommonRuntimeError> {
        let mut store = Store::new(&self.engine, context);

        let (module, instance) =
            Module::instantiate_async(&mut store, &self.component, &self.linker)
                .await
                .map_err(|error| {
                    CommonRuntimeError::ModuleInstantiationFailed(format!("{error}"))
                })?;

        NativeFunction::new(self.definition.clone(), store, module, instance)
    }
}
