use crate::{
    backends::{self, EngineBackend, InstanceBackend, ModuleBackend},
    vm::VirtualMachine,
    HostCallbackFn, Result,
};
use ct_common::{ModuleDefinition, ModuleId};

/// A [`Runtime`] creates [`Module`]s.
pub struct Runtime {
    inner: backends::Engine,
}

impl Runtime {
    /// Create a new [`Runtime`].
    pub fn new(callback: impl HostCallbackFn) -> Result<Self> {
        let inner = backends::Engine::new(callback, vec![VirtualMachine::JavaScript])?;
        Ok(Runtime { inner })
    }

    /// Construct a new [`Module`] factory given a `definition`.
    pub fn module(&self, definition: ModuleDefinition) -> Result<Module> {
        let id: ModuleId = (&definition).into();
        Ok(Module::new(id, self.inner.module(definition)?))
    }
}

/// A [`Module`] creates [`Instance`]s.
pub struct Module {
    inner: backends::Module,
    id: ModuleId,
}

impl Module {
    fn new(id: ModuleId, inner: backends::Module) -> Self {
        Self { id, inner }
    }

    /// Get the [`ModuleId`] for this module.
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    /// Create a new [`Instance`] of this module.
    pub fn instantiate(&mut self) -> Result<Instance> {
        Ok(Instance::new(self.inner.instantiate()?))
    }
}

/// An [`Instance`] can execute sandboxed code.
pub struct Instance {
    inner: backends::Instance,
}

impl Instance {
    fn new(inner: backends::Instance) -> Self {
        Self { inner }
    }

    /// Invoke this instance.
    pub fn run(&mut self, input: Option<String>) -> Result<Option<String>> {
        self.inner.run(input)
    }
}
