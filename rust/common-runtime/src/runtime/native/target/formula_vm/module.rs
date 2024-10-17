use crate::{
    module::HasModuleContext,
    target::formula_vm::{NativeFormulaVmContext, VirtualModule},
    CommonRuntimeError, FormulaVmDefinition, HasModuleContextMut, Module, ModuleId,
    ModuleInstanceId,
};
use std::sync::Arc;
use wasmtime::{AsContextMut, Store};

use super::{Datom, Instruction, RangeQuery, Scalar, State};

/// An `common:formula/virtual-module`-based Module for the
/// [crate::NativeRuntime].
pub struct NativeFormulaVm {
    instance_id: ModuleInstanceId,
    module_id: ModuleId,
    store: Store<NativeFormulaVmContext>,
    module: VirtualModule,
}

impl NativeFormulaVm {
    /// Instantiate a [NativeFormulaVm] with a [crate::ModuleDefinition] and
    /// other Wasm runtime-specific acoutrement
    pub fn new(
        definition: Arc<FormulaVmDefinition>,
        store: Store<NativeFormulaVmContext>,
        module: VirtualModule,
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

impl NativeFormulaVm {
    /// `init` function in VM.
    pub async fn init(
        &mut self,
        input: &[(String, Scalar)],
    ) -> Result<(State, RangeQuery), CommonRuntimeError> {
        let (state, query) = self
            .module
            .common_formula_module()
            .call_init(self.store.as_context_mut(), input)
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;
        Ok((state, query))
    }
    /// `step` function in VM.
    pub async fn step(
        &mut self,
        state: &State,
        datoms: Vec<Datom>,
    ) -> Result<(State, Vec<Instruction>), CommonRuntimeError> {
        let (state, instructions) = self
            .module
            .common_formula_module()
            .call_step(self.store.as_context_mut(), state, &datoms)
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;
        Ok((state, instructions))
    }

    /// `end` function in VM.
    pub async fn end(&mut self, state: &State) -> Result<Vec<Instruction>, CommonRuntimeError> {
        let instructions = self
            .module
            .common_formula_module()
            .call_end(self.store.as_context_mut(), state)
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(error.to_string()))?;
        Ok(instructions)
    }
}

impl Module for NativeFormulaVm {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn instance_id(&self) -> &ModuleInstanceId {
        &self.instance_id
    }
}

impl HasModuleContext for NativeFormulaVm {
    type Context = NativeFormulaVmContext;

    fn context(&self) -> &Self::Context {
        self.store.data()
    }
}

impl HasModuleContextMut for NativeFormulaVm {
    fn context_mut(&mut self) -> &mut Self::Context {
        self.store.data_mut()
    }
}
