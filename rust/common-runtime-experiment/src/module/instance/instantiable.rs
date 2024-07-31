use crate::CommonRuntimeError;
use async_trait::async_trait;

use super::{ModuleInstance, Runtime};

mod local_function;
mod local_function_vm;
mod remote_function;
mod remote_function_vm;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Instantiable<Rt>
where
    Rt: Runtime,
{
    type ModuleInstance: ModuleInstance<Rt>;

    async fn instantiate(
        &mut self,
        runtime: Rt,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError>;
}
