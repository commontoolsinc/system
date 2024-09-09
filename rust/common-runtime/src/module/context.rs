use crate::{sync::ConditionalSend, InputOutput};
use common_ifc::Context as IfcContext;

/// The basic properties that are required to define the host-to-guest bindings
/// for a given Common Module
pub trait ModuleContext: ConditionalSend {
    /// The type of [`InputOutput`] in use by the [`ModuleContext`]
    type Io: InputOutput;

    /// Read access to the [`ModuleContext`]'s [`InputOutput`]
    fn io(&self) -> &Self::Io;

    /// Read access to the [`ModuleContext`]'s [`IfcContext`]
    fn ifc(&self) -> &IfcContext;
}

/// Mutable properties that may be required in a [`ModuleContext`] in order to
/// make use of some Common Modules
pub trait ModuleContextMut: ModuleContext {
    /// Read-write access to the [`ModuleContext`]'s [`InputOutput`]
    fn io_mut(&mut self) -> &mut Self::Io;
}

/// A trait that is implemented by things that have a [`ModuleContext`]. All
/// Common Modules implement this trait.
pub trait HasModuleContext {
    /// The type of the [`ModuleContext`] that backs the implementor
    type Context: ModuleContext;

    /// Read access to the [`ModuleContext`] that backs the implementor
    fn context(&self) -> &Self::Context;
}

/// A trait that is implemented when a [`HasModuleContext`] allows mutable
/// access to its backing [`ModuleContext`]
pub trait HasModuleContextMut: HasModuleContext {
    /// Mutable access to the [`ModuleContext`] that backs the implementor
    fn context_mut(&mut self) -> &mut Self::Context;
}

impl<T> HasModuleContext for &T
where
    T: HasModuleContext,
{
    type Context = T::Context;

    fn context(&self) -> &Self::Context {
        (**self).context()
    }
}

impl<T> HasModuleContext for &mut T
where
    T: HasModuleContext,
{
    type Context = T::Context;

    fn context(&self) -> &Self::Context {
        (**self).context()
    }
}
