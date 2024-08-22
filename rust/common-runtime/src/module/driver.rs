use async_trait::async_trait;

use crate::CommonRuntimeError;

use super::ModuleFactory;

/// A [ModuleDriver] is implemented by a Runtime for each distinctive form of
/// [crate::ModuleDefinition] that it is able to instantiate. The driver
/// produces a prepared [ModuleFactory] for a given [crate::ModuleDefinition],
/// which can in turn be used to instantiate a [crate::Module].
#[async_trait]
pub trait ModuleDriver<D> {
    /// The type of the [ModuleFactory] that is produced by this [ModuleDriver]
    type ModuleFactory: ModuleFactory;

    /// Prepare a Module, producing a [ModuleFactory] that may be used to
    /// instantiate an associated [crate::Module] many times (relatively
    /// cheaply).
    ///
    /// Preparation typically entails resolving a Wasm artifact for the Module
    /// body, compiling that artifact and caching all the metadata required to
    /// instantiate it.
    async fn prepare(&self, definition: D) -> Result<Self::ModuleFactory, CommonRuntimeError>;
}
