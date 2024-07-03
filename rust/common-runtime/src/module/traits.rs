use std::collections::BTreeMap;

use crate::SourceCode;
use async_trait::async_trait;
use bytes::Bytes;
use common_wit::Target;

use crate::{sync::ConditionalSync, CommonRuntimeError, InputOutput};

/// A [Module] embodies the substance of a Common Module.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Module: Clone + ConditionalSync {
    /// Reports the [Target] of the [Module], which implies the shape of its exported API
    fn target(&self) -> Target;

    /// Reports a unique ID for the [Module]
    async fn id(&self) -> Result<&str, CommonRuntimeError>;
}

/// A trait that may be implemented by anything that can be converted to Wasm
/// Component bytes
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToWasmComponent: Module {
    /// Performs the conversion to Wasm Component bytes
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError>;
}

/// A trait that may be implemented by anything that can be converted to the
/// interior sources from which a Common Module is derived
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToModuleSources: Module {
    /// Performs the conversion to Common Module sources
    async fn to_module_sources(
        &self,
    ) -> Result<Option<BTreeMap<String, SourceCode>>, CommonRuntimeError>;
}

/// A [PreparedModule] is a [Module] that is ready to be invoked.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait PreparedModule: Clone + ConditionalSync {
    /// Invoke the [PreparedModule] against the provided [InputOutput] by
    /// calling into its interior implementation of `call`.
    async fn call(
        &self,
        io: Box<dyn InputOutput>,
    ) -> Result<Box<dyn InputOutput>, CommonRuntimeError>;
}

/// A [ModulePreparer] is able to convert any [Module] into a particular
/// implementation of a [PreparedModule].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModulePreparer<Mod: Module + 'static>: Clone + ConditionalSync {
    /// The type of [PreparedModule] that is produced by this [ModulePreparer]
    type PreparedModule: PreparedModule;

    /// Prepare a [Module], which amounts to performing whatever
    /// transformation, compilation and/or pre-initialization that may be
    /// necessaary for optimal runtime qualities.
    async fn prepare(&mut self, module: Mod) -> Result<Self::PreparedModule, CommonRuntimeError>;
}
