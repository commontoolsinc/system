use super::{super::function_bindings::module::Module as GuestModule, NativeFunctionContext};
use crate::{
    module::{FunctionInterface, HasModuleContext, ModuleContext},
    CommonRuntimeError, FunctionDefinition, HasModuleContextMut, InputOutput, IoData, Module,
    ModuleContextMut, ModuleId, ModuleInstanceId, Validated,
};
use async_trait::async_trait;
use std::sync::Arc;
use wasmtime::{AsContextMut, Store};

/// An `common:function/module`-based Module for the [crate::NativeRuntime].
pub struct NativeFunction {
    module_id: ModuleId,
    instance_id: ModuleInstanceId,
    store: Store<NativeFunctionContext>,
    module: GuestModule,
}

impl NativeFunction {
    /// Instantiate a [NativeFunction] with a [crate::ModuleDefinition] and
    /// other Wasm runtime-specific acoutrement
    pub fn new(
        definition: Arc<FunctionDefinition>,
        store: Store<NativeFunctionContext>,
        module: GuestModule,
    ) -> Result<Self, CommonRuntimeError> {
        let module_id = ModuleId::from(&*(*definition));
        let instance_id = ModuleInstanceId::try_from(module_id.clone())?;

        Ok(Self {
            module_id,
            instance_id,
            store,
            module,
        })
    }
}

#[async_trait]
impl FunctionInterface for NativeFunction {
    type InputOutput = <NativeFunctionContext as ModuleContext>::Io;

    #[instrument(skip(self, io))]
    async fn run(
        &mut self,
        io: Validated<Self::InputOutput>,
    ) -> Result<IoData, CommonRuntimeError> {
        debug!("Running the module...");
        let mut io = io.into_inner();

        std::mem::swap(self.context_mut().io_mut(), &mut io);

        self.module
            .call_run(self.store.as_context_mut())
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;

        Ok(std::mem::take(self.context_mut().io_mut().output_mut()))
    }
}

impl Module for NativeFunction {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn instance_id(&self) -> &ModuleInstanceId {
        &self.instance_id
    }
}

impl HasModuleContext for NativeFunction {
    type Context = NativeFunctionContext;

    fn context(&self) -> &Self::Context {
        self.store.data()
    }
}

impl HasModuleContextMut for NativeFunction {
    fn context_mut(&mut self) -> &mut Self::Context {
        self.store.data_mut()
    }
}
