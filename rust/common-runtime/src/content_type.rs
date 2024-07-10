/// Supported content types that may be embodied as a [crate::CommonModule]
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    /// JavaScript or TypeScript code
    JavaScript,
    /// Python code
    Python,
}
