use async_trait::async_trait;

use crate::{sync::ConditionalSync, CommonRuntimeError};

use super::{Module, ModuleContext};

/// A [ModuleFactory] constitutes a prepared Module, which may be instantiated
/// many times relatively cheaply.
///
/// [ModuleFactory]s are typically produced by an implementor of a
/// [crate::ModuleDriver].
#[async_trait]
pub trait ModuleFactory: Clone + ConditionalSync {
    /// The type of the backing [ModuleContext] that is used by this [ModuleFactory]
    type Context: ModuleContext;
    /// The type of the [Module] that is instantiated by this [ModuleFactory]
    type Module: Module<Context = Self::Context>;

    /// Given a [ModuleContext], instantiate the [Module] that this
    /// [ModuleFactory] represents
    async fn instantiate(&self, context: Self::Context)
        -> Result<Self::Module, CommonRuntimeError>;
}
