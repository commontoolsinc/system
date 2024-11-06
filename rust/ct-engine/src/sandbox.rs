use crate::{Error, Result};
use ct_common::{ModuleDefinition, ModuleId};
use ct_runtime::{HostCallbackFn, Instance, Module, Runtime};
use std::collections::HashMap;

/// Manages [`Module`] and [`Instance`] instances and lifetimes.
///
/// Currently, each module has a single instance used for
/// all invocations.
pub struct SandboxManager {
    runtime: Runtime,
    modules: HashMap<ModuleId, Module>,
    instances: HashMap<ModuleId, Instance>,
}

impl SandboxManager {
    pub fn new(callback: impl HostCallbackFn) -> Result<Self> {
        let runtime = Runtime::new(callback)?;
        Ok(Self {
            runtime,
            modules: Default::default(),
            instances: Default::default(),
        })
    }

    pub fn define(&mut self, definition: ModuleDefinition) -> Result<ModuleId> {
        let id = (&definition).into();
        if self.modules.contains_key(&id) {
            return Ok(id);
        }
        let module = self.runtime.module(definition)?;
        self.modules.insert(id.clone(), module);
        Ok(id)
    }

    pub fn run(&mut self, id: &ModuleId, input: String) -> Result<String> {
        // We use a single instance per module for now
        let instance = if let Some(instance) = self.instances.get_mut(id) {
            instance
        } else {
            // No instance yet, create one if module defined
            let module = self
                .modules
                .get_mut(id)
                .ok_or_else(|| Error::ModuleNotFound(id.to_owned()))?;
            let instance = module.instantiate()?;
            self.instances.insert(id.to_owned(), instance);
            self.instances
                .get_mut(id)
                .ok_or_else(|| Error::ModuleNotFound(id.to_owned()))?
        };
        Ok(instance.run(input)?)
    }
}
