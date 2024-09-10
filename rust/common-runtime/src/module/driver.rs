use async_trait::async_trait;

use crate::CommonRuntimeError;

use super::ModuleFactory;

#[cfg(doc)]
use crate::{Module, ModuleDefinition};

/// Runtime Module Drivers.
///
/// A [`ModuleDriver`] is implemented by a Runtime for each distinctive form of
/// [`ModuleDefinition`] that it is able to instantiate. The driver
/// produces a prepared [`ModuleFactory`] for a given [`ModuleDefinition`],
/// which can in turn be used to instantiate a [`Module`].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModuleDriver<D> {
    /// The type of the [`ModuleFactory`] that is produced by this
    /// [`ModuleDriver`].
    type ModuleFactory: ModuleFactory;

    /// Prepare a Module, producing a [`ModuleFactory`] that may be used
    /// to instantiate an associated [`Module`] many times (relatively cheaply).
    /// Preparation typically entails resolving a Wasm artifact for the
    /// Module body, compiling that artifact and caching all the metadata
    /// required to instantiate it.
    async fn prepare(&self, definition: D) -> Result<Self::ModuleFactory, CommonRuntimeError>;
}
