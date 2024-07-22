use crate::{
    wasmtime::WasmtimePreparedScript, CommonRuntimeError, ModuleDefinition, ModuleId,
    ModulePreparer, ToModuleSources, ToWasmComponent,
};
use async_trait::async_trait;
use common_wit::InputOutput;
use sieve_cache::SieveCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::{
    component::{Component, Linker},
    Engine,
};

use super::WasmtimeInterpreterModule;

/// A [WasmtimeInterpreter] prepares an appropriate interpreter (provided as a
/// Wasm Component) and loads it with the source files of a Common Module in
/// order to run that Module without having to first compile it into a separate
/// Wasm Component. The beneficial trade-off is that the runtime speed of the
/// Common Module is slower than it could be, but the amount of time required to
/// begin execution is much shorter than compiling it.
#[derive(Clone)]
pub struct WasmtimeInterpreter<Io>
where
    Io: InputOutput,
{
    engine: Engine,
    prepared_interpreters: Arc<Mutex<SieveCache<ModuleId, Arc<WasmtimeInterpreterModule<Io>>>>>,
}

impl<Io> WasmtimeInterpreter<Io>
where
    Io: InputOutput,
{
    /// Instantiate a [WasmtimeInterpreter]
    pub fn new(engine: Engine) -> Result<Self, CommonRuntimeError> {
        Ok(Self {
            engine,
            prepared_interpreters: Arc::new(Mutex::new(
                SieveCache::new(64)
                    .map_err(|error| CommonRuntimeError::SandboxCreationFailed(error.into()))?,
            )),
        })
    }
}

#[async_trait]
impl<Module, Io> ModulePreparer<Module> for WasmtimeInterpreter<Io>
where
    Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    Io: InputOutput,
{
    type PreparedModule = WasmtimePreparedScript<Io>;

    #[instrument(skip(self, module), fields(module.target = %module.target()))]
    async fn prepare(
        &mut self,
        module: Module,
    ) -> Result<Self::PreparedModule, CommonRuntimeError> {
        debug!("Checking cache for prepared module...");

        let module_id = module.id().await?;
        let has_interpreter = {
            self.prepared_interpreters
                .lock()
                .await
                .contains_key(&module_id)
        };

        if !has_interpreter {
            debug!("No prepared interpreter found in cache; preparing...");

            let interpreter_wasm = module.to_wasm_component().await?;

            let component = Component::new(&self.engine, interpreter_wasm)
                .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?;

            let mut linker = Linker::new(&self.engine);

            common_bindings::link_common_script(&mut linker)
                .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;
            common_bindings::link_wasi_async(&mut linker)
                .map_err(|error| CommonRuntimeError::LinkFailed(format!("{error}")))?;

            self.prepared_interpreters.lock().await.insert(
                module_id.clone(),
                Arc::new(WasmtimeInterpreterModule::new(
                    module_id.clone(),
                    self.engine.clone(),
                    linker,
                    component,
                )),
            );

            debug!("Interpreter is now cached...");
        } else {
            debug!("Interpreter already cached!")
        }

        debug!("Retrieving interpreter from cache...");

        let interpreter = self
            .prepared_interpreters
            .lock()
            .await
            .get(&module_id)
            .ok_or_else(|| {
                CommonRuntimeError::PreparationFailed(
                    "Prepared interpreter unexpectedly missing from cache".into(),
                )
            })?
            .clone();

        let sources = module
            .to_module_sources()
            .await?
            .ok_or_else(|| CommonRuntimeError::PreparationFailed("No sources provided".into()))?;

        let Some(source_code) = sources.get("module") else {
            return Err(CommonRuntimeError::PreparationFailed(
                "No script source provided".into(),
            ));
        };

        Ok(WasmtimePreparedScript {
            source: String::from_utf8_lossy(&source_code.body).to_string(),
            interpreter,
        })
    }
}
