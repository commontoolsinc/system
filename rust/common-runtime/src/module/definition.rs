use super::ModuleId;
use crate::IoShape;
use common_wit::Target;

use super::{affinity::Affinity, body::ModuleBody};

mod function;
pub use function::*;

mod function_vm;
pub use function_vm::*;

mod remote_function;
pub use remote_function::*;

#[cfg(doc)]
use crate::{Module, ModuleDriver};

/// A (de)serializable structure that constitutes a [`Module`].
///
/// A [`ModuleDefinition`] can be instantiated as a live Module by a Runtime that
/// implements an appropriate [`ModuleDriver`].
#[derive(Clone)]
pub struct ModuleDefinition {
    /// The [`Target`] of the Module, which is always dereferencable to a WIT
    /// definition
    pub target: Target,
    /// The [`Affinity`] of the Module, which informs the Runtime where the Module
    /// ought to be scheduled relative to the local device
    pub affinity: Affinity,
    /// The shape of the input keys that are able to be read by the Module
    pub inputs: IoShape,
    /// The shape of the output keys that are able to be written-to by the Module
    pub outputs: IoShape,
    /// The [`ModuleBody`] represents the substantive implementation of the Module, either
    /// as inputs that may be used to derive a runnable Wasm artifact
    pub body: ModuleBody,
}

impl From<&ModuleDefinition> for ModuleId {
    fn from(value: &ModuleDefinition) -> Self {
        (&value.body).into()
    }
}
