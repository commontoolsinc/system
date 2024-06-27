use crate::BuilderError;
use async_trait::async_trait;
use bytes::Bytes;

/// A trait to build a WASM artifact containing WIT modules, and
/// source code to be executed within the artifact, where specific
/// implementations of [Bake] provide a runtime to execute that source.
#[async_trait]
pub trait Bake {
    /// Build a WASM artifact containing the WIT modules and means
    /// to execute `source_code`.
    async fn bake(
        &self,
        world: &str,
        wit: Vec<Bytes>,
        source_code: Bytes,
        library: Vec<Bytes>,
    ) -> Result<Bytes, BuilderError>;
}
