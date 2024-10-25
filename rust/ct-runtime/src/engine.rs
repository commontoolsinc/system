use crate::{ModuleDefinition, Result};
use async_trait::async_trait;

/// Interface of backends to provide the of
/// the runtime implementation.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Engine {
    /// Concrete type of [`Module`] produced by this engine.
    type Module: Module;
    /// Create a [`Module`] factory derived from the [`ModuleDefinition`].
    async fn module(&self, definition: ModuleDefinition) -> Result<Self::Module>;
}

/// A factory to create executable instances.
///
/// A [`Module`] is a static description of a
/// Common Process. Instances of this module can be
/// instantiated. Lifetimes TBD.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Module {
    /// Concrete type of [`Instance`] produced by this module.
    type Instance: Instance;
    /// Instantiate a new [`Instance`].
    async fn instantiate(&mut self) -> Result<Self::Instance>;
}

/// An active instance of a [`Module`].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Instance {
    /// Run the process in this instance.
    async fn run(&mut self, input: String) -> Result<String>;
}
