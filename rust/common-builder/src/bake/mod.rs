mod fs;
mod javascript;
mod python;
mod r#trait;

use common_wit::Target;
pub use javascript::*;
pub use python::*;
pub use r#trait::*;

use async_trait::async_trait;
use bytes::Bytes;

/// Multivariant implementation of [`Bake`].
pub enum Baker {
    /// Uses [`JavaScriptBaker`].
    JavaScript,
    /// Uses [`PythonBaker`].
    Python,
}

#[async_trait]
impl Bake for Baker {
    async fn bake(&self, target: Target, source_code: Bytes) -> Result<Bytes, crate::BuilderError> {
        match self {
            Baker::JavaScript => (JavaScriptBaker {}).bake(target, source_code).await,
            Baker::Python => (PythonBaker {}).bake(target, source_code).await,
        }
    }
}
