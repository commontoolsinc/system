use std::collections::BTreeMap;

use async_trait::async_trait;
use blake3::Hash;
use bytes::Bytes;

use crate::{sync::ConditionalSync, CommonRuntimeError, ContentType, InputOutput};

/// A [CommonModule] embodies the substance of a Common Module.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CommonModule: Clone + ConditionalSync {
    /// Reports the interior [ContentType] of the [CommonModule]
    fn content_type(&self) -> ContentType;

    /// Reports the hash of the interior sources of the [CommonModule], which
    /// should uniquely identify its sources among all possible modules.
    fn hash(&self) -> Hash;
}

/// A [PreparedModule] is a [CommonModule] that is ready to be invoked.
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

/// A [ModulePreparer] is able to convert any [CommonModule] into a particular
/// implementation of a [PreparedModule].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModulePreparer<Module: CommonModule + 'static>: Clone + ConditionalSync {
    /// The type of [PreparedModule] that is produced by this [ModulePreparer]
    type PreparedModule: PreparedModule;

    /// Prepare a [CommonModule], which amounts to performing whatever
    /// transformation, compilation and/or pre-initialization that may be
    /// necessaary for optimal runtime qualities.
    async fn prepare(&mut self, module: Module)
        -> Result<Self::PreparedModule, CommonRuntimeError>;
}

/// A trait that may be implemented by anything that can be converted to Wasm
/// Component bytes
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToWasmComponent {
    /// Performs the conversion to Wasm Component bytes
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError>;
}

/// A pairing of raw source code bytes and an associated [ContentType]
pub struct ModuleSource {
    /// The mime of the source
    pub content_type: ContentType,
    /// The raw bytes of the source
    pub source: Bytes,
}

/// A trait that may be implemented by anything that can be converted to the
/// interior sources from which a Common Module is derived
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ToModuleSources {
    /// Performs the conversion to Common Module sources
    async fn to_module_sources(
        &self,
    ) -> Result<Option<BTreeMap<String, ModuleSource>>, CommonRuntimeError>;
}
