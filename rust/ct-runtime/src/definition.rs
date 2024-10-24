use crate::{Error, VirtualMachine};

/// A description of a [`Module`].
pub struct ModuleDefinition {
    /// Type of [`VirtualMachine`] to interpret `source`.
    pub vm: VirtualMachine,
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
            vm: VirtualMachine::JavaScript,
        }
    }
}

/// A struct uniquely identifying a [`ModuleDefinition`].
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ModuleId(blake3::Hash);

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ModuleId {
    type Err = Error;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(
            blake3::Hash::from_str(s).map_err(|e| Error::from(e.to_string()))?,
        ))
    }
}

impl From<&ModuleDefinition> for ModuleId {
    fn from(value: &ModuleDefinition) -> ModuleId {
        let mut hasher = blake3::Hasher::new();

        hasher.update(value.vm.as_bytes());
        hasher.update(value.source.as_bytes());

        ModuleId(hasher.finalize())
    }
}
