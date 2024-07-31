use common_wit::Target;

use crate::{InputOutput, ModuleBody, ModuleInstance, PreparedModule, Runtime};
use async_trait::async_trait;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait NativeMachine {
    type PreparedModule: PreparedModule;

    async fn prepare_compiled_module(
        &mut self,
        target: Target,
        body: ModuleBody,
    ) -> Self::PreparedModule;
}

pub trait VirtualMachine {}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait LocalInstantiation<Rt>
where
    Rt: Runtime,
{
    type InputOutput: InputOutput;
    type ModuleInstance: ModuleInstance<Rt>;

    async fn instantiate_local_module<P>(&mut self, prepared_module: P) -> Self::ModuleInstance
    where
        P: PreparedModule<InputOutput = Self::InputOutput, ModuleInstance = Self::ModuleInstance>;
}

pub trait RemoteInstantiation {}
