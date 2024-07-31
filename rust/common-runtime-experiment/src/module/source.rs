use crate::ContentType;
use bytes::Bytes;
use std::collections::BTreeMap;

/// A pairing of raw source code bytes and an associated [ContentType]
#[derive(Debug, Clone)]
pub struct SourceCode {
    /// The mime of the source
    pub content_type: ContentType,
    /// The raw bytes of the source
    pub body: Bytes,
}

pub type SourceCodeCollection = BTreeMap<String, SourceCode>;
