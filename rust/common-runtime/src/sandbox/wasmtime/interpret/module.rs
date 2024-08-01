use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::{
    component::{Component, Instance, Linker},
    AsContextMut, Engine, Store,
};

use crate::{
    wasmtime::bindings::common_function_vm::VirtualModule, CommonRuntimeError, InputOutput,
    ModuleId, ModuleInstance, ModuleInstanceId, PreparedModule,
};

use super::bindings::FunctionVmHostState;

/// An instantiated interpreter than may be "loaded" with the source code of a
/// Common Module in the language that the interpreter supports.
pub struct WasmtimeLiveScript<Io>
where
    Io: InputOutput,
{
    id: ModuleInstanceId,

    // TODO: Synchronization wrapper may not be needed after we stub wasi:*
    store: Arc<Mutex<Store<FunctionVmHostState<Io>>>>,
    virtual_module: VirtualModule,

    // REASON: Instance must be retained until module is dropped
    #[allow(dead_code)]
    instance: Instance,
}

impl<Io> WasmtimeLiveScript<Io>
where
    Io: InputOutput,
{
    async fn set_source(&mut self, source: &str) -> Result<(), CommonRuntimeError> {
        self.virtual_module
            .call_set_source(self.store.lock().await.as_context_mut(), source)
            .await
            .map_err(|error| CommonRuntimeError::ModuleInstantiationFailed(format!("{error}")))?
            .map_err(|error| {
                CommonRuntimeError::ModuleInstantiationFailed(format!("Script error: {error}"))
            })
    }
}

#[async_trait]
impl<Io> ModuleInstance for WasmtimeLiveScript<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    fn id(&self) -> &ModuleInstanceId {
        &self.id
    }

    async fn run(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::InputOutput, crate::CommonRuntimeError> {
        let mut store = self.store.lock().await;

        store.data_mut().replace_io(io);

        self.virtual_module
            .call_run(store.as_context_mut())
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;

        Ok(store.data_mut().take_io())
    }
}

/// A pairing of a compiled interpreter and a source script that should be run
/// within that interpreter. This construct is an envelope to make instantiating
/// a loading a script into an interpreter easy to do in one step.
#[derive(Clone)]
pub struct WasmtimePreparedScript<Io>
where
    Io: InputOutput,
{
    /// The source code to be interpreted
    pub source: String,
    /// The compiled (but not yet instantiated) interpreter
    pub interpreter: Arc<WasmtimeInterpreterModule<Io>>,
}

#[async_trait]
impl<Io> PreparedModule for WasmtimePreparedScript<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    type ModuleInstance = WasmtimeLiveScript<Io>;

    async fn instantiate(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
        let mut live_script = self.interpreter.instantiate(io).await?;
        live_script.set_source(&self.source).await?;
        Ok(live_script)
    }
}

/// A compiled (but not yet instantiated) Wasm Component that implements an
/// interpreter for a given source language.
#[derive(Clone)]
pub struct WasmtimeInterpreterModule<Io>
where
    Io: InputOutput,
{
    id: ModuleId,

    // TODO(#36): There is a re-factor that will let us share a lot of
    // overlapping implementation with the compiler; we need to make the
    // compiler generic over host bindings so that we can re-spawn it
    // interpreter mode.
    engine: Engine,
    linker: Linker<FunctionVmHostState<Io>>,
    component: Component,
}

impl<Io> WasmtimeInterpreterModule<Io>
where
    Io: InputOutput,
{
    /// Initialize the [WasmtimeInterpreterModule]
    pub fn new(
        id: ModuleId,
        engine: Engine,
        linker: Linker<FunctionVmHostState<Io>>,
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
impl<Io> PreparedModule for WasmtimeInterpreterModule<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    type ModuleInstance = WasmtimeLiveScript<Io>;

    async fn instantiate(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
        let mut store = Store::new(&self.engine, FunctionVmHostState::new(io));

        let (common, instance) =
            VirtualModule::instantiate_async(&mut store, &self.component, &self.linker)
                .await
                .map_err(|error| {
                    CommonRuntimeError::ModuleInstantiationFailed(format!("{error}"))
                })?;

        Ok(WasmtimeLiveScript {
            id: self.id.clone().try_into()?,
            store: Arc::new(Mutex::new(store)),
            virtual_module: common,
            instance,
        })
    }
}
