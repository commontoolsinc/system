use super::FunctionInstance;
use crate::CommonRuntimeError;

use async_trait::async_trait;

pub struct RemoteFunction {}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FunctionInstance for RemoteFunction {
    async fn run(&self) -> Result<(), CommonRuntimeError> {
        todo!()
    }
}

pub struct RemoteFunctionVm {}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FunctionInstance for RemoteFunctionVm {
    async fn run(&self) -> Result<(), CommonRuntimeError> {
        todo!()
    }
}
