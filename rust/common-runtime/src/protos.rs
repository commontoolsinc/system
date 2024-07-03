#![allow(missing_docs)]

pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

pub mod common {
    tonic::include_proto!("common");
}

pub mod builder {
    tonic::include_proto!("builder");
}

pub mod runtime {
    tonic::include_proto!("runtime");
}

use crate::{ContentType, ModuleSource, SourceCode};
use common_wit::Target;

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
