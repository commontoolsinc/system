use async_trait::async_trait;

use crate::{
    affinity::LocalOnly,
    runtime::feature::{LocalInstantiation, NativeMachine},
    target::CommonFunction,
    CommonRuntimeError, LocalFunction, Module, Runtime,
};

use super::Instantiable;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<Rt> Instantiable<Rt> for Module<CommonFunction, LocalOnly>
where
    Rt: Runtime + LocalInstantiation + NativeMachine + 'static,
{
    type ModuleInstance = LocalFunction<Rt>;

    async fn instantiate(
        &mut self,
        runtime: Rt,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
        todo!()
    }
}

// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
// impl<R> Instantiable for Module<CommonFunction, LocalOnly>
// where
//     R: LocalInstantiation + NativeMachine,
// {
//     type ModuleInstance = LocalFunction<R>;

//     async fn instantiate(
//         &mut self,
//         runtime: R,
//     ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
//         todo!()
//     }
// }
