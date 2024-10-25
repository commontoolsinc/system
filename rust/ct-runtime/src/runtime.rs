use crate::{vm::VirtualMachine, Engine, HostFeatures, Result};

#[cfg(doc)]
use crate::Module;

#[cfg(not(target_arch = "wasm32"))]
type InnerEngine<T> = crate::backends::WasmtimeEngine<T>;
#[cfg(target_arch = "wasm32")]
type InnerEngine<T> = crate::backends::WclEngine<T>;

/// A description of a [`Module`].
pub struct ModuleDefinition {
    /// Type of [`VirtualMachine`] to interpret `source`.
    pub vm: VirtualMachine,
    /// Source code to execute in `vm`.
    pub source: String,
}

/// A [`Runtime`] creates [`Engine::Module`]s
pub struct Runtime<H: HostFeatures> {
    engine: InnerEngine<H>,
}

impl<H> Runtime<H>
where
    H: HostFeatures,
{
    /// Create a new [`Runtime`].
    pub fn new() -> Result<Self> {
        let engine = InnerEngine::new(vec![VirtualMachine::JavaScript])?;
        Ok(Runtime { engine })
    }

    /// Construct a new [`Module`] factory given a `definition`.
    pub fn module(
        &self,
        definition: ModuleDefinition,
    ) -> Result<<InnerEngine<H> as Engine>::Module> {
        self.engine.module(definition)
    }
}
