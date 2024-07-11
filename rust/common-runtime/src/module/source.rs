use crate::ContentType;
use bytes::Bytes;
use common_protos::common;
use common_wit::Target;
use std::collections::BTreeMap;

/// A structured collection of source inputs needed to build a module with the given [WitTarget].
#[derive(Debug, Clone)]
pub struct ModuleSource {
    /// The target that the source inputs implement
    pub target: Target,

    /// A mapping of unique name to input [SourceCode]
    pub source_code: BTreeMap<String, SourceCode>,
}

/// A pairing of raw source code bytes and an associated [ContentType]
#[derive(Debug, Clone)]
pub struct SourceCode {
    /// The mime of the source
    pub content_type: ContentType,
    /// The raw bytes of the source
    pub body: Bytes,
}

impl From<common::ModuleSource> for ModuleSource {
    fn from(value: common::ModuleSource) -> Self {
        ModuleSource {
            target: match value.target() {
                common::Target::CommonModule => Target::CommonModule,
            },
            source_code: value
                .source_code
                .into_iter()
                .map(|(name, source_code)| (name, source_code.into()))
                .collect(),
        }
    }
}

impl From<ModuleSource> for common::ModuleSource {
    fn from(value: ModuleSource) -> Self {
        common::ModuleSource {
            target: match value.target {
                Target::CommonModule => common::Target::CommonModule.into(),
            },
            source_code: value
                .source_code
                .into_iter()
                .map(|(name, source_code)| (name, source_code.into()))
                .collect(),
        }
    }
}

impl From<common::SourceCode> for SourceCode {
    fn from(value: common::SourceCode) -> Self {
        SourceCode {
            content_type: ContentType::from(value.content_type()),
            body: value.body.into(),
        }
    }
}

impl From<SourceCode> for common::SourceCode {
    fn from(value: SourceCode) -> Self {
        common::SourceCode {
            content_type: common::ContentType::from(value.content_type).into(),
            body: value.body.into(),
        }
    }
}
