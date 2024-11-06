use crate::Result;
use ct_common::ModuleDefinition;

/// Interface of backends to provide the of
/// the runtime implementation.
pub trait EngineBackend {
    /// Concrete type of [`ModuleBackend`] produced by this engine.
    type Module: ModuleBackend;
    /// Create a [`ModuleBackend`] factory derived from the [`ModuleDefinition`].
    fn module(&self, definition: ModuleDefinition) -> Result<Self::Module>;
}

/// A factory to create executable instances.
///
/// A [`ModuleBackend`] is a static description of a
/// Common Process. Instances of this module can be
/// instantiated. Lifetimes TBD.
pub trait ModuleBackend {
    /// Concrete type of [`InstanceBackend`] produced by this module.
    type Instance: InstanceBackend;
    /// Instantiate a new [`InstanceBackend`].
    fn instantiate(&mut self) -> Result<Self::Instance>;
}

/// An active instance of a [`ModuleBackend`].
pub trait InstanceBackend {
    /// Run the process in this instance.
    fn run(&mut self, input: String) -> Result<String>;
}
