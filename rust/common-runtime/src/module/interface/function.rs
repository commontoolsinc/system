use async_trait::async_trait;

use crate::{CommonRuntimeError, InputOutput, IoData, Module, Validated};

/// A trait that defines the host-side interface over an implementor of
/// `common:function` Module.
#[async_trait]
pub trait FunctionInterface: Module {
    /// The type of [InputOutput] that is expected by the underlying Module
    /// implementation
    type InputOutput: InputOutput;

    /// Invoke `run` on the guest `common:function`, substituting the provided
    /// [InputOutput] within the Module's execution context.
    async fn run(&mut self, io: Validated<Self::InputOutput>)
        -> Result<IoData, CommonRuntimeError>;
}
