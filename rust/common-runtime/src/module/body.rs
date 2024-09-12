use super::{source_code::SourceCodeCollection, ModuleId, SourceCode};
use crate::{CommonRuntimeError, Result};
use common_protos::common as protos;

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

impl From<&ModuleBody> for ModuleId {
    fn from(value: &ModuleBody) -> ModuleId {
        match value {
            ModuleBody::Signature(id) => id.clone(),
            ModuleBody::SourceCode(source_code_collection) => {
                let mut hasher = blake3::Hasher::new();

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

impl TryFrom<protos::ModuleBody> for ModuleBody {
    type Error = CommonRuntimeError;
    fn try_from(value: protos::ModuleBody) -> Result<Self> {
        let variant = value.variant.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match variant {
            protos::module_body::Variant::ModuleSignature(module_signature) => {
                ModuleBody::Signature(ModuleId::Base64(module_signature.id.clone()))
            }
            protos::module_body::Variant::ModuleSource(module_source) => ModuleBody::SourceCode(
                module_source
                    .source_code
                    .into_iter()
                    .map(|(key, value)| (key, SourceCode::from(value)))
                    .collect(),
            ),
        })
    }
}

impl From<ModuleBody> for protos::ModuleBody {
    fn from(value: ModuleBody) -> Self {
        let variant = match value {
            ModuleBody::Signature(id) => {
                protos::module_body::Variant::ModuleSignature(protos::ModuleSignature {
                    id: id.to_string(),
                })
            }
            ModuleBody::SourceCode(source_code_collection) => {
                protos::module_body::Variant::ModuleSource(protos::ModuleSource {
                    source_code: source_code_collection
                        .into_iter()
                        .map(|(name, source_code)| (name, source_code.into()))
                        .collect(),
                })
            }
        };

        protos::ModuleBody {
            variant: Some(variant),
        }
    }
}
