use crate::{CommonRuntimeError, ConditionalSync, InputOutput};

use async_trait::async_trait;

use super::ModuleInstance;

/// A [PreparedModule] is a [crate::Module] that is ready to be instantiated. For example,
/// an underlying Wasm Component may be compiled but not yet instanced.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait PreparedModule: Clone + ConditionalSync {
    /// An implementation of [InputOutput] that will be used to provide
    /// state to the instantiated module
    type InputOutput: InputOutput;

    /// The shape of the module after instantiation
    type ModuleInstance: ModuleInstance<InputOutput = Self::InputOutput>;

    /// Instantiate the [PreparedModule], using the provided [InputOutput] as
    /// the "default" state shared with the module instance.
    async fn instantiate(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<T> PreparedModule for std::sync::Arc<T>
where
    T: PreparedModule,
{
    type InputOutput = T::InputOutput;

    type ModuleInstance = T::ModuleInstance;

    async fn instantiate(
        &self,
        io: Self::InputOutput,
    ) -> Result<Self::ModuleInstance, CommonRuntimeError> {
        T::instantiate(self, io).await
    }
}
