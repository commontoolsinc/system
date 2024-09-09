use common_protos::common::{self, ModuleSource};
use common_protos::runtime::instantiate_module_request::ModuleReference;
use common_wit::Target;

use super::ModuleId;

use super::source_code::SourceCodeCollection;

/// The variants that are accepted as the body field of a
/// [crate::ModuleDefinition].
#[derive(Clone)]
pub enum ModuleBody {
    /// A signature body is a [`ModuleId`] that can be used to look up a pre-built
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

impl ModuleBody {
    // TODO: Module reference should not be a holder of target
    /// Convert the [`ModuleBody`] into a [`ModuleReference`] with the given
    /// [`Target`]
    pub fn to_module_reference(&self, target: &Target) -> ModuleReference {
        let target = common::Target::from(target);
        match self {
            ModuleBody::Signature(id) => {
                ModuleReference::ModuleSignature(common::ModuleSignature {
                    target: target.into(),
                    id: id.to_string(),
                })
            }
            ModuleBody::SourceCode(source_code_collection) => {
                ModuleReference::ModuleSource(ModuleSource {
                    target: target.into(),
                    source_code: source_code_collection
                        .iter()
                        .map(|(name, source_code)| (name.to_owned(), source_code.into()))
                        .collect(),
                })
            }
        }
    }
}
