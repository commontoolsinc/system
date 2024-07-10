use std::sync::Arc;

use async_trait::async_trait;
use sieve_cache::SieveCache;
use tokio::sync::Mutex;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, OptLevel,
};

use crate::{
    wasmtime::bindings::Common, CommonRuntimeError, InputOutput, ModuleDefinition, ModuleId,
    ModulePreparer, ToWasmComponent,
};

use super::WasmtimeCompiledModule;

/// A [WasmtimeBuilder] prepares a [CommonModule] by converting the full set of
/// sources into a single Wasm Component. The first time this is done for a
/// unique set of input sources, it entails an relatively expensive compilation
/// and assembly process. This cost is amortized across all successive
/// preparations. The beneficial trade-off is that every subsequent execution of
/// the module over time (even across sessions) may be significantly faster than
/// other options (that may entail e.g., interpreting the code).
#[derive(Clone)]
pub struct WasmtimeCompiler<Io>
where
    Io: InputOutput,
{
    engine: Engine,
    prepared_modules: Arc<Mutex<SieveCache<ModuleId, Arc<WasmtimeCompiledModule<Io>>>>>,
}

impl<Io> WasmtimeCompiler<Io>
where
    Io: InputOutput,
{
    /// Instantiate a [WasmtimeCompiler]
    pub fn new() -> Result<Self, CommonRuntimeError> {
        let mut config = Config::default();

        config.cranelift_opt_level(OptLevel::Speed);
        config.async_support(false);

        let engine = Engine::new(&config)
            .map_err(|error| CommonRuntimeError::SandboxCreationFailed(format!("{error}")))?;

        Ok(Self {
            engine,
            prepared_modules: Arc::new(Mutex::new(
                SieveCache::new(64)
                    .map_err(|error| CommonRuntimeError::SandboxCreationFailed(error.into()))?,
            )),
        })
    }
}

#[async_trait]
impl<Module, Io> ModulePreparer<Module> for WasmtimeCompiler<Io>
where
    Module: ModuleDefinition + ToWasmComponent + 'static,
    Io: InputOutput,
{
    type PreparedModule = Arc<WasmtimeCompiledModule<Io>>;

    #[instrument(skip(self, module), fields(module.target = %module.target()))]
    async fn prepare(
        &mut self,
        module: Module,
    ) -> Result<Self::PreparedModule, CommonRuntimeError> {
        debug!("Checking cache for prepared module...");

        let module_id = module.id().await?;
        let has_module = { self.prepared_modules.lock().await.contains_key(module_id) };

        if !has_module {
            debug!("No prepared module found in cache; preparing...");
            let component_wasm = module.to_wasm_component().await?;

            let component = Component::new(&self.engine, component_wasm)
                .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?;

            let mut linker = Linker::new(&self.engine);

            wasmtime_wasi::add_to_linker_sync(&mut linker)
                .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

            wasmtime_wasi_http::proxy::sync::add_only_http_to_linker(&mut linker)
                .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

            Common::add_to_linker(&mut linker, |environment| environment)
                .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

            self.prepared_modules.lock().await.insert(
                module_id.clone(),
                Arc::new(WasmtimeCompiledModule::new(
                    module_id.clone(),
                    self.engine.clone(),
                    linker,
                    component,
                )),
            );

            debug!("Module is now cached...");
        } else {
            debug!("Module already cached!")
        }

        debug!("Retrieving module from cache...");
        self.prepared_modules
            .lock()
            .await
            .get(module_id)
            .ok_or_else(|| {
                CommonRuntimeError::PreparationFailed(
                    "Prepared module unexpectedly missing from cache".into(),
                )
            })
            .cloned()
    }
}
