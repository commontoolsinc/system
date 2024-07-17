use std::collections::BTreeMap;

use crate::{ModuleId, ModuleInstanceId, SourceCode};
use async_trait::async_trait;
use bytes::Bytes;
use common_wit::Target;

use crate::{sync::ConditionalSync, CommonRuntimeError, InputOutput};

/// A [ModuleDefinition] embodies the substance of a Common Module.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModuleDefinition: std::fmt::Debug + Clone + ConditionalSync {
    /// Reports the [Target] of the [ModuleDefinition], which implies the shape of its exported API
    fn target(&self) -> Target;

    /// Reports a unique ID for the [ModuleDefinition]
    async fn id(&self) -> Result<ModuleId, CommonRuntimeError>;
}

/// A trait that may be implemented by anything that can be converted to Wasm
/// Component bytes
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToWasmComponent: ModuleDefinition {
    /// Performs the conversion to Wasm Component bytes
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError>;
}

/// A trait that may be implemented by anything that can be converted to the
/// interior sources from which a Common Module is derived
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToModuleSources: ModuleDefinition {
    /// Performs the conversion to Common Module sources
    async fn to_module_sources(
        &self,
    ) -> Result<Option<BTreeMap<String, SourceCode>>, CommonRuntimeError>;
}

/// A [ModulePreparer] is able to convert any [ModuleDefinition] into a particular
/// implementation of a [PreparedModule].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModulePreparer<Module: ModuleDefinition + 'static>: Clone + ConditionalSync {
    /// The type of [PreparedModule] that is produced by this [ModulePreparer]
    type PreparedModule: PreparedModule;

    /// Prepare a [ModuleDefinition], which amounts to performing whatever
    /// transformation, compilation and/or pre-initialization that may be
    /// necessaary for optimal runtime qualities.
    async fn prepare(&mut self, module: Module)
        -> Result<Self::PreparedModule, CommonRuntimeError>;
}

/// A [PreparedModule] is a [ModuleDefinition] that is ready to be instantiated. For example,
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

/// A [ModuleInstance] represents a live instantiation of a Common Module. The
/// Common Module's `run` implementation may be invoked, allowing it to perform
/// some changes to a provided [InputOutput].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModuleInstance: ConditionalSync {
    /// An implementation of [InputOutput] that will be used to provide state to
    /// the instantiated module
    type InputOutput: InputOutput;

    /// The uniquely-identifying ID of this [ModuleInstance]
    fn id(&self) -> &ModuleInstanceId;

    /// Invoke the exported `run` function on the inner Common Module
    /// implementation, assigning the provided [InputOutput] to its local state
    /// in advance.
    async fn run(&self, io: Self::InputOutput) -> Result<Self::InputOutput, CommonRuntimeError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<T> ModuleInstance for Box<T>
where
    T: ModuleInstance,
{
    type InputOutput = T::InputOutput;

    fn id(&self) -> &ModuleInstanceId {
        T::id(self)
    }

    async fn run(&self, io: Self::InputOutput) -> Result<Self::InputOutput, CommonRuntimeError> {
        T::run(self, io).await
    }
}
