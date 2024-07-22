// REASON: Docs will be added when stubs are filled out
#![allow(missing_docs)]
use async_trait::async_trait;
use std::marker::PhantomData;

use crate::{ModuleDefinition, ModuleInstance, ModulePreparer, PreparedModule, ToWasmComponent};
use commit_wit::InputOutput;

#[derive(Clone)]
pub struct CompiledModuleInstance<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

impl<Io> CompiledModuleInstance<Io>
where
    Io: InputOutput,
{
    pub fn new() -> Self {
        CompiledModuleInstance {
            _marker: PhantomData,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Io> ModuleInstance for CompiledModuleInstance<Io>
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
pub struct CompilerPreparedModule<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Io> PreparedModule for CompilerPreparedModule<Io>
where
    Io: InputOutput,
{
    type InputOutput = Io;

    type ModuleInstance = CompiledModuleInstance<Io>;

    async fn instantiate(
        &self,
        _io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, crate::CommonRuntimeError> {
        todo!()
    }
}

#[derive(Clone)]
pub struct BrowserCompiler<Io>
where
    Io: InputOutput,
{
    _marker: PhantomData<Io>,
}

impl<Io> BrowserCompiler<Io>
where
    Io: InputOutput,
{
    pub fn new() -> Self {
        BrowserCompiler {
            _marker: PhantomData,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Module, Io> ModulePreparer<Module> for BrowserCompiler<Io>
where
    Module: ModuleDefinition + ToWasmComponent + 'static,
    Io: InputOutput,
{
    type PreparedModule = CompilerPreparedModule<Io>;

    async fn prepare(
        &mut self,
        _module: Module,
    ) -> Result<Self::PreparedModule, crate::CommonRuntimeError> {
        todo!()
    }
}
