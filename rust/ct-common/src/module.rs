/// The content type of a module.
pub enum ContentType {
    /// The JavaScript language.
    JavaScript,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ContentType::JavaScript => "JavaScript",
            }
        )
    }
}

/// A description of a module.
pub struct ModuleDefinition {
    /// The language of `source`.
    pub content_type: ContentType,
    /// Source code to execute in `vm`.
    pub source: String,
}

impl<T> From<T> for ModuleDefinition
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        ModuleDefinition {
            source: value.into(),
            content_type: ContentType::JavaScript,
        }
    }
}

/// A hash uniquely identifying a [`ModuleDefinition`].
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ModuleId(blake3::Hash);

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ModuleId {
    type Err = String;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s).map_err(|e| e.to_string())?))
    }
}

impl From<&ModuleDefinition> for ModuleId {
    fn from(value: &ModuleDefinition) -> ModuleId {
        let mut hasher = blake3::Hasher::new();

        hasher.update(value.content_type.to_string().as_bytes());
        hasher.update(value.source.as_bytes());

        ModuleId(hasher.finalize())
    }
}
