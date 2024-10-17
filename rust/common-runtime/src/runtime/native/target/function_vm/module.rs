use super::{
    super::function_bindings::vm::VirtualModule as GuestVirtualModule, NativeFunctionVmContext,
};
use crate::{
    module::{FunctionInterface, HasModuleContext, ModuleContext},
    CommonRuntimeError, FunctionVmDefinition, HasModuleContextMut, InputOutput, IoData, Module,
    ModuleContextMut, ModuleId, ModuleInstanceId, Validated,
};
use async_trait::async_trait;
use std::sync::Arc;
use wasmtime::{AsContextMut, Store};

/// An `common:function/virtual-module`-based Module for the
/// [crate::NativeRuntime].
pub struct NativeFunctionVm {
    instance_id: ModuleInstanceId,
    module_id: ModuleId,
    store: Store<NativeFunctionVmContext>,
    module: GuestVirtualModule,
}

impl NativeFunctionVm {
    /// Instantiate a [NativeFunctionVm] with a [crate::ModuleDefinition] and
    /// other Wasm runtime-specific acoutrement
    pub fn new(
        definition: Arc<FunctionVmDefinition>,
        store: Store<NativeFunctionVmContext>,
        module: GuestVirtualModule,
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
impl FunctionInterface for NativeFunctionVm {
    type InputOutput = <NativeFunctionVmContext as ModuleContext>::Io;

    async fn run(
        &mut self,
        io: Validated<Self::InputOutput>,
    ) -> Result<IoData, CommonRuntimeError> {
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

impl Module for NativeFunctionVm {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn instance_id(&self) -> &ModuleInstanceId {
        &self.instance_id
    }
}

impl HasModuleContext for NativeFunctionVm {
    type Context = NativeFunctionVmContext;

    fn context(&self) -> &Self::Context {
        self.store.data()
    }
}

impl HasModuleContextMut for NativeFunctionVm {
    fn context_mut(&mut self) -> &mut Self::Context {
        self.store.data_mut()
    }
}
