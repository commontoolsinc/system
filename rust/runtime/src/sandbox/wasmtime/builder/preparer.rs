use async_trait::async_trait;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, OptLevel,
};

use crate::{
    wasmtime::bindings::Common, CommonModule, CommonRuntimeError, ModulePreparer, ToWasmComponent,
};

use super::WasmtimePrebuiltModule;

/// A [WasmtimeBuilder] prepares a [CommonModule] by converting the full set of
/// sources into a single Wasm Component. The first time this is done for a
/// unique set of input sources, it entails an relatively expensive compilation
/// and assembly process. This cost is amortized across all successive
/// preparations. The beneficial trade-off is that every subsequent execution of
/// the module over time (even across sessions) may be significantly faster than
/// other options (that may entail e.g., interpreting the code).
#[derive(Clone)]
pub struct WasmtimeBuilder {
    engine: Engine,
}

impl WasmtimeBuilder {
    /// Instantiate a [WasmtimeBuilder]
    pub fn new() -> Result<Self, CommonRuntimeError> {
        let mut config = Config::default();

        config.cranelift_opt_level(OptLevel::Speed);
        config.async_support(false);

        let engine = Engine::new(&config)
            .map_err(|error| CommonRuntimeError::SandboxCreationFailed(format!("{error}")))?;

        Ok(Self { engine })
    }
}

#[async_trait]
impl<Module> ModulePreparer<Module> for WasmtimeBuilder
where
    Module: CommonModule + ToWasmComponent + 'static,
{
    type PreparedModule = WasmtimePrebuiltModule;

    async fn prepare(
        &mut self,
        module: Module,
    ) -> Result<Self::PreparedModule, CommonRuntimeError> {
        let component_wasm = module.to_wasm_component().await?;

        let component = Component::new(&self.engine, component_wasm)
            .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?;

        let mut linker = Linker::new(&self.engine);

        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        Common::add_to_linker(&mut linker, |environment| environment)
            .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

        Ok(WasmtimePrebuiltModule::new(
            self.engine.clone(),
            linker,
            component,
        ))
    }
}
