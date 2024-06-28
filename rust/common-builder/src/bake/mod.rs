mod baker;
mod fs;
mod javascript;
mod python;

pub use baker::*;
pub use javascript::*;
pub use python::*;

use async_trait::async_trait;
use bytes::Bytes;

/// Multivariant implementation of [Bake].
pub enum Baker {
    /// Uses [JavaScriptBaker].
    JavaScript,
    /// Uses [PythonBaker].
    Python,
}

#[async_trait]
impl Bakerlike for Baker {
    async fn bake(
        &self,
        world: &str,
        wit: Vec<Bytes>,
        source_code: Bytes,
        library: Vec<Bytes>,
    ) -> Result<Bytes, crate::BuilderError> {
        match self {
            Baker::JavaScript => {
                (JavaScriptBaker {})
                    .bake(world, wit, source_code, library)
                    .await
            }
            Baker::Python => {
                (PythonBaker {})
                    .bake(world, wit, source_code, library)
                    .await
            }
        }
    }
}
