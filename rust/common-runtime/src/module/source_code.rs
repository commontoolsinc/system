use crate::ContentType;
use bytes::Bytes;
use common_protos::common;
use std::collections::BTreeMap;

/// A pairing of raw source code bytes and an associated [`ContentType`]
#[derive(Debug, Clone)]
pub struct SourceCode {
    /// The mime of the source
    pub content_type: ContentType,
    /// The raw bytes of the source
    pub body: Bytes,
}

/// A mapping of human-readable module specifiers to source code files
pub type SourceCodeCollection = BTreeMap<String, SourceCode>;

impl From<common::SourceCode> for SourceCode {
    fn from(value: common::SourceCode) -> Self {
        SourceCode {
            content_type: ContentType::from(value.content_type()),
            body: value.body.into(),
        }
    }
}

impl From<&SourceCode> for common::SourceCode {
    fn from(value: &SourceCode) -> Self {
        common::SourceCode {
            content_type: common::ContentType::from(value.content_type).into(),
            body: value.body.clone().into(),
        }
    }
}

impl From<SourceCode> for common::SourceCode {
    fn from(value: SourceCode) -> Self {
        (&value).into()
    }
}
