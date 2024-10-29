use crate::{ModuleDefinition, Result};

/// Interface of backends to provide the of
/// the runtime implementation.
pub trait Engine {
    /// Concrete type of [`Module`] produced by this engine.
    type Module: Module;
    /// Create a [`Module`] factory derived from the [`ModuleDefinition`].
    fn module(&self, definition: ModuleDefinition) -> Result<Self::Module>;
}

/// A factory to create executable instances.
///
/// A [`Module`] is a static description of a
/// Common Process. Instances of this module can be
/// instantiated. Lifetimes TBD.
pub trait Module {
    /// Concrete type of [`Instance`] produced by this module.
    type Instance: Instance;
    /// Instantiate a new [`Instance`].
    fn instantiate(&mut self) -> Result<Self::Instance>;
}

/// An active instance of a [`Module`].
pub trait Instance {
    /// Run the process in this instance.
    fn run(&mut self, input: String) -> Result<String>;
}
