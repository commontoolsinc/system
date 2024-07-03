use super::protos;

/// Supported content types that may be embodied as a [crate::CommonModule]
#[derive(Clone, Copy)]
pub enum ContentType {
    /// JavaScript or TypeScript code
    JavaScript,
    /// Python code
    Python,
}

impl From<protos::common::ContentType> for ContentType {
    fn from(value: protos::common::ContentType) -> Self {
        match value {
            protos::common::ContentType::JavaScript => ContentType::JavaScript,
            protos::common::ContentType::Python => ContentType::Python,
        }
    }
}

impl From<ContentType> for protos::common::ContentType {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::JavaScript => protos::common::ContentType::JavaScript,
            ContentType::Python => protos::common::ContentType::Python,
        }
    }
}
