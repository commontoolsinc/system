use crate::{Error, Result};
use ranked_prolly_tree::Key as KeyTrait;

const INVALID_KEY_LENGTH: &str = "Key components must all be 32 bytes.";

/// Key type in the [`CtStorage`],
/// a concatenation of 32-byte hashes
/// of an entity, namespace, and attribute.
#[derive(Clone, Debug)]
pub struct Key([u8; 96]);

impl Key {
    /// Create a new [`Key`] from UTF8 strings.
    pub fn new(entity: &str, ns: &str, attr: &str) -> Self {
        Self::from_components(
            hash(entity.as_bytes()),
            hash(ns.as_bytes()),
            hash(attr.as_bytes()),
        )
    }

    /// Create a new [`Key`] from already encoded hashes.
    pub fn from_components(entity: [u8; 32], ns: [u8; 32], attr: [u8; 32]) -> Self {
        let mut key = [0; 96];
        key[0..32].copy_from_slice(&entity);
        key[32..64].copy_from_slice(&ns);
        key[64..96].copy_from_slice(&attr);
        Key(key)
    }

    /// Create a new [`Key`] from already encoded hashes.
    /// Fails if total length is not 96.
    pub fn from_slices(entity: &[u8], ns: &[u8], attr: &[u8]) -> Result<Self> {
        if entity.len() != 32 || ns.len() != 32 || attr.len() != 32 {
            return Err(Error::Internal(INVALID_KEY_LENGTH.into()));
        }
        let mut key = [0; 96];
        key[0..32].copy_from_slice(&entity);
        key[32..64].copy_from_slice(&ns);
        key[64..96].copy_from_slice(&attr);
        Ok(Key(key))
    }

    /// Returns the entity component of the key as a hash.
    pub fn entity(&self) -> &[u8] {
        &self.0[0..32]
    }

    /// Returns the namespace component of the key as a hash.
    pub fn ns(&self) -> &[u8] {
        &self.0[32..64]
    }

    /// Returns the attribute component of the key as a hash.
    pub fn attr(&self) -> &[u8] {
        &self.0[64..96]
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.entity()
            .cmp(other.entity())
            .then(self.ns().cmp(other.ns()))
            .then(self.attr().cmp(other.attr()))
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Key {}

impl KeyTrait for Key {}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Vec<u8>> for Key {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        if value.len() != 96 {
            return Err(Error::Internal(INVALID_KEY_LENGTH.into()));
        }
        Key::from_slices(&value[0..32], &value[32..64], &value[64..96])
    }
}

fn hash(input: &[u8]) -> [u8; 32] {
    <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(input))
}
