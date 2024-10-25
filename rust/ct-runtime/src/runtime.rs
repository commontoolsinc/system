use crate::{vm::VirtualMachine, Engine, Result};

#[cfg(doc)]
use crate::Module;

#[cfg(not(target_arch = "wasm32"))]
type InnerEngine = crate::backends::WasmtimeEngine;
#[cfg(target_arch = "wasm32")]
type InnerEngine = crate::backends::WclEngine;

/// A description of a [`Module`].
pub struct ModuleDefinition {
    /// Type of [`VirtualMachine`] to interpret `source`.
    pub vm: VirtualMachine,
    /// Source code to execute in `vm`.
    pub source: String,
}

/// A [`Runtime`] creates [`Engine::Module`]s
pub struct Runtime {
    engine: InnerEngine,
}

impl Runtime {
    /// Create a new [`Runtime`].
    pub fn new() -> Result<Self> {
        let engine = InnerEngine::new(vec![VirtualMachine::JavaScript])?;
        Ok(Runtime { engine })
    }

    /// Construct a new [`Module`] factory given a `definition`.
    pub async fn module(
        &self,
        definition: ModuleDefinition,
    ) -> Result<<InnerEngine as Engine>::Module> {
        self.engine.module(definition).await
    }
}
