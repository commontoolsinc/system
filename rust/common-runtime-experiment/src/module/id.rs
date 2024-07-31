use std::{fmt::Display, str::FromStr};

use crate::CommonRuntimeError;

/// A unique instance ID that may be used to identify an instantiation of an
/// given module.
///
/// TODO: Would be nice to say that an instance ID was derived from a module ID
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleInstanceId(pub String);

impl Display for ModuleInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A [ModuleId] uniquely identifies a given Common Module. At this time it is
/// always expected to represent the hash of the Common Module's Wasm Component
/// artifact.
///
/// TODO: This needs a definition that supports interpreted mode. Hash of
/// sources, perhaps?
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ModuleId {
    /// The [ModuleId] is in the form of [blake3::Hash] bytes.
    Hash(blake3::Hash),
    /// The [ModuleId] is a base64-encoded string
    Base64(String),
}

impl From<String> for ModuleId {
    fn from(value: String) -> Self {
        ModuleId::Base64(value)
    }
}

impl From<blake3::Hash> for ModuleId {
    fn from(value: blake3::Hash) -> Self {
        ModuleId::Hash(value)
    }
}

impl TryFrom<ModuleId> for blake3::Hash {
    type Error = CommonRuntimeError;

    fn try_from(value: ModuleId) -> Result<Self, Self::Error> {
        Ok(match value {
            ModuleId::Hash(hash) => hash,
            ModuleId::Base64(string) => blake3::Hash::from_str(&string)
                .map_err(|error| CommonRuntimeError::InvalidModuleId(format!("{error}")))?,
        })
    }
}

impl TryFrom<ModuleId> for ModuleInstanceId {
    type Error = CommonRuntimeError;

    fn try_from(value: ModuleId) -> Result<Self, Self::Error> {
        let millis = std::time::SystemTime::now()
            .elapsed()
            .map_err(|error| CommonRuntimeError::InternalError(format!("{error}")))?
            .as_millis();

        let entropy = rand::random::<u64>();
        let hash = blake3::Hash::try_from(value)?;
        let mut hasher = blake3::Hasher::new_keyed(hash.as_bytes());

        Ok(ModuleInstanceId(
            hasher
                .update(
                    &millis
                        .to_le_bytes()
                        .into_iter()
                        .chain(entropy.to_le_bytes())
                        .collect::<Vec<u8>>(),
                )
                .finalize()
                .to_string(),
        ))
    }
}

impl ModuleId {
    /// Convert the [ModuleId] to raw bytes corresponding to the [blake3::Hash]
    pub fn to_bytes(&self) -> Result<Vec<u8>, CommonRuntimeError> {
        Ok(match self {
            ModuleId::Hash(hash) => hash.as_bytes().to_vec(),
            ModuleId::Base64(string) => blake3::Hash::from_str(string)
                .map_err(|error| CommonRuntimeError::InvalidModuleId(format!("{error}")))?
                .as_bytes()
                .to_vec(),
        })
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleId::Hash(hash) => write!(f, "{}", hash),
            ModuleId::Base64(string) => write!(f, "{string}"),
        }
    }
}
