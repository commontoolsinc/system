use std::fmt::Display;

use common_protos::common;

/// Supported content types that may be embodied as a [crate::CommonModule]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// JavaScript or TypeScript code
    JavaScript,
    /// Python code
    Python,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ContentType::JavaScript => "text/javascript",
                ContentType::Python => "text/x-python",
            }
        )
    }
}

impl From<common::ContentType> for ContentType {
    fn from(value: common::ContentType) -> Self {
        match value {
            common::ContentType::JavaScript => ContentType::JavaScript,
            common::ContentType::Python => ContentType::Python,
        }
    }
}

impl From<ContentType> for common::ContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::JavaScript => common::ContentType::JavaScript,
            ContentType::Python => common::ContentType::Python,
        }
    }
}
