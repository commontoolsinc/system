use async_trait::async_trait;
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};

use crate::{wasmtime::bindings::Common, CommonRuntimeError, InputOutput, PreparedModule};

use super::bindings::ModuleHostState;

/// A [WasmtimePrebuiltModule] is a pre-transformed, pre-compiled Common Module.
/// In other words: fully-self-contained Wasm Component bytes.
#[derive(Clone)]
pub struct WasmtimePrebuiltModule {
    engine: Engine,
    linker: Linker<ModuleHostState>,
    component: Component,
}

impl WasmtimePrebuiltModule {
    /// Initialize the [WasmtimePrebuiltModule]
    pub fn new(engine: Engine, linker: Linker<ModuleHostState>, component: Component) -> Self {
        Self {
            engine,
            linker,
            component,
        }
    }
}

#[async_trait]
impl PreparedModule for WasmtimePrebuiltModule {
    async fn call(
        &self,
        io: Box<dyn InputOutput>,
    ) -> Result<Box<dyn InputOutput>, CommonRuntimeError> {
        let mut store = Store::new(&self.engine, ModuleHostState::new(io.into()));

        let (common, _inst) = Common::instantiate(&mut store, &self.component, &self.linker)
            .map_err(|error| CommonRuntimeError::ModuleInstantiationFailed(format!("{error}")))?;

        let common_module = common.common_module_module();

        match common_module.call_create(&mut store) {
            Ok(body_resource) => {
                common
                    .common_module_module()
                    .body()
                    .call_run(&mut store, body_resource)
                    .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?;
            }
            Err(error) => {
                return Err(CommonRuntimeError::ModuleInstantiationFailed(format!(
                    "{error}"
                )))
            }
        };

        Ok(store.into_data().take_io().into())
    }
}
