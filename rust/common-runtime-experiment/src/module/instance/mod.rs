mod instantiable;
pub use instantiable::*;

mod function;
pub use function::*;

use super::ModuleInstanceId;
use crate::{ConditionalSync, InputOutput};

pub trait Runtime {
    type InputOutput: InputOutput;
}

/// A [ModuleInstance] represents a live instantiation of a Common Module. The
/// Common Module's `run` implementation may be invoked, allowing it to perform
/// some changes to a provided [InputOutput].
pub trait ModuleInstance<Rt>: ConditionalSync
where
    Rt: Runtime,
{
    /// The uniquely-identifying ID of this [ModuleInstance]
    fn id(&self) -> &ModuleInstanceId;

    fn io(&self) -> &Rt::InputOutput;
}

// impl<T> ModuleInstance for Box<T>
// where
//     T: ModuleInstance,
// {
//     type InputOutput = T::InputOutput;

//     fn id(&self) -> &ModuleInstanceId {
//         T::id(self)
//     }
// }
