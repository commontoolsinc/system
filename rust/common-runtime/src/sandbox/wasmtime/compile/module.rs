use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use wasmtime::{
    component::{Component, Instance, Linker},
    AsContextMut, Engine, Store,
};

use crate::{
    wasmtime::bindings::common_module::Common, CommonRuntimeError, InputOutput, ModuleId,
    ModuleInstance, ModuleInstanceId, PreparedModule,
};

use super::bindings::ModuleHostState;

/// A [WasmtimeCompiledModule] is a pre-transformed, pre-compiled Common Module.
/// In other words: a harness to run fully-self-contained Wasm Component bytes.
#[derive(Clone)]
pub struct WasmtimeCompiledModule<Io>
where
    Io: InputOutput,
{
    id: ModuleId,
    engine: Engine,
    linker: Linker<ModuleHostState<Io>>,
    component: Component,
}

impl<Io> WasmtimeCompiledModule<Io>
where
    Io: InputOutput,
{
    /// Initialize the [WasmtimeCompiledModule]
    pub fn new(
        id: ModuleId,
        engine: Engine,
        linker: Linker<ModuleHostState<Io>>,
        component: Component,
    ) -> Self {
        Self {
            id,
            engine,
            linker,
            component,
        }
    }
}

#[async_trait]
impl<Io> PreparedModule for WasmtimeCompiledModule<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;
    type ModuleInstance = WasmtimeModuleInstance<Io>;

    async fn instantiate(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
        let mut store = Store::new(&self.engine, ModuleHostState::new(io));

        let (common, instance) =
            Common::instantiate_async(&mut store, &self.component, &self.linker)
                .await
                .map_err(|error| {
                    CommonRuntimeError::ModuleInstantiationFailed(format!("{error}"))
                })?;

        Ok(WasmtimeModuleInstance {
            id: self.id.clone().try_into()?,
            store: Arc::new(Mutex::new(store)),
            common,
            instance,
        })
    }
}

/// A live Common Module as instantiated by a [crate::wasmtime::WasmtimeCompiler]
pub struct WasmtimeModuleInstance<Io>
where
    Io: InputOutput,
{
    id: ModuleInstanceId,
    // TODO: Synchronization wrapper may not be needed after we stub wasi:*
    store: Arc<Mutex<Store<ModuleHostState<Io>>>>,
    common: Common,

    // REASON: Instance must be retained until module is dropped
    #[allow(dead_code)]
    instance: Instance,
}

#[async_trait]
impl<Io> ModuleInstance for WasmtimeModuleInstance<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    async fn run(&self, io: Self::InputOutput) -> Result<Self::InputOutput, CommonRuntimeError> {
        let mut store = self.store.lock().await;

        store.data_mut().replace_io(io);

        self.common
            .call_run(store.as_context_mut())
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;

        Ok(store.data_mut().take_io())
    }

    fn id(&self) -> &ModuleInstanceId {
        &self.id
    }
}
