use common_wit::Target;

use super::ModuleId;

use super::source_code::SourceCodeCollection;

/// The variants that are accepted as the body field of a
/// [crate::ModuleDefinition].
#[derive(Clone)]
pub enum ModuleBody {
    /// A signature body is a [ModuleId] that can be used to look up a pre-built
    /// artifact
    Signature(ModuleId),
    /// A source code body contains code that may be built into a runnable
    /// artifact
    SourceCode(SourceCodeCollection),
}

impl From<(&Target, &ModuleBody)> for ModuleId {
    fn from((target, value): (&Target, &ModuleBody)) -> ModuleId {
        match value {
            ModuleBody::Signature(id) => id.clone(),
            ModuleBody::SourceCode(source_code_collection) => {
                let mut hasher = blake3::Hasher::new();

                hasher.update(target.to_string().as_bytes());

                for (name, source_code) in source_code_collection {
                    hasher.update(name.as_bytes());
                    hasher.update(source_code.content_type.to_string().as_bytes());
                    hasher.update(source_code.body.as_ref());
                }

                ModuleId::Hash(hasher.finalize())
            }
        }
    }
}
