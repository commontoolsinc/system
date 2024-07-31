mod remote;
pub use remote::*;

mod local;
pub use local::*;

use async_trait::async_trait;

use crate::CommonRuntimeError;

use super::{ModuleInstance, Runtime};

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait FunctionInstance<R>: ModuleInstance<R>
where
    R: Runtime,
{
    async fn run(&self, io: R::InputOutput) -> Result<R::InputOutput, CommonRuntimeError>;
}
