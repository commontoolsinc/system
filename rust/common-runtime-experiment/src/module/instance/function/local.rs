use std::marker::PhantomData;

use super::FunctionInstance;
use crate::{CommonRuntimeError, InputOutput, ModuleInstance, ModuleInstanceId, Runtime};
use async_trait::async_trait;

pub struct LocalFunction<Rt>
where
    Rt: Runtime,
{
    id: ModuleInstanceId,
    io: Rt::InputOutput,
}

impl<Rt> ModuleInstance<Rt> for LocalFunction<Rt>
where
    Rt: Runtime,
{
    fn id(&self) -> &ModuleInstanceId {
        &self.id
    }

    fn io(&self) -> &<Rt as Runtime>::InputOutput {
        &self.io
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Rt> FunctionInstance<Rt> for LocalFunction<Rt>
where
    Rt: Runtime,
{
    async fn run(&self) -> Result<(), CommonRuntimeError> {
        todo!()
    }
}

pub struct LocalFunctionVm {}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FunctionInstance for LocalFunctionVm {
    async fn run(&self) -> Result<(), CommonRuntimeError> {
        todo!()
    }
}
