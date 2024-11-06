use crate::{sandbox::SandboxManager, Result};
use ct_common::{ModuleDefinition, ModuleId};
use ct_runtime::HostCallbackFn;

pub struct Engine {
    sandbox: SandboxManager,
}

impl Engine {
    pub fn new(host_callback: impl HostCallbackFn) -> Result<Self> {
        Ok(Engine {
            sandbox: SandboxManager::new(host_callback)?,
        })
    }

    pub fn define(&mut self, definition: ModuleDefinition) -> Result<ModuleId> {
        self.sandbox.define(definition)
    }

    pub fn run(&mut self, id: &ModuleId, input: String) -> Result<String> {
        self.sandbox.run(id, input)
    }
}
