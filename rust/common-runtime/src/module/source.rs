use std::collections::BTreeMap;

use bytes::Bytes;
use common_wit::Target;

use crate::ContentType;

/// A structured collection of source inputs needed to build a module with the given [WitTarget].
#[derive(Clone)]
pub struct ModuleSource {
    /// The target that the source inputs implement
    pub target: Target,

    /// A mapping of unique name to input [SourceCode]
    pub source_code: BTreeMap<String, SourceCode>,
}

/// A pairing of raw source code bytes and an associated [ContentType]
#[derive(Clone)]
pub struct SourceCode {
    /// The mime of the source
    pub content_type: ContentType,
    /// The raw bytes of the source
    pub body: Bytes,
}
