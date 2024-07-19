// REASON: Docs will be added when stubs are filled out
#![allow(missing_docs)]

use async_trait::async_trait;
use std::marker::PhantomData;

use crate::{
    InputOutput, ModuleDefinition, ModuleInstance, ModulePreparer, PreparedModule, ToModuleSources,
    ToWasmComponent,
};

#[derive(Clone)]
pub struct InterpretedModuleInstance<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

impl<Io> InterpretedModuleInstance<Io>
where
    Io: InputOutput,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Io> ModuleInstance for InterpretedModuleInstance<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    fn id(&self) -> &crate::ModuleInstanceId {
        todo!()
    }

    async fn run(
        &self,
        _io: Self::InputOutput,
    ) -> Result<Self::InputOutput, crate::CommonRuntimeError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct InterpreterPreparedModule<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

impl<Io> InterpreterPreparedModule<Io>
where
    Io: InputOutput,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Io> PreparedModule for InterpreterPreparedModule<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    type ModuleInstance = InterpretedModuleInstance<Io>;

    async fn instantiate(
        &self,
        _io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, crate::CommonRuntimeError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct BrowserInterpreter<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

impl<Io> BrowserInterpreter<Io>
where
    Io: InputOutput,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Module, Io> ModulePreparer<Module> for BrowserInterpreter<Io>
where
    Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    Io: InputOutput,
{
    type PreparedModule = InterpreterPreparedModule<Io>;

    async fn prepare(
        &mut self,
        _module: Module,
    ) -> Result<Self::PreparedModule, crate::CommonRuntimeError> {
        todo!()
    }
}
