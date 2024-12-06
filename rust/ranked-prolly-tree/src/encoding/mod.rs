use crate::{Error, Result};
use async_trait::async_trait;
use ct_common::ConditionalSync;
use serde::{de::DeserializeOwned, Serialize};

/// Type of hash produced by an [`Encoder`].
pub type Hash = Vec<u8>;
/// Reference to a [`Hash`].
pub type HashRef = <Hash as std::ops::Deref>::Target;

/// A helper utility to provide [`std::fmt::Display`] for a [`Hash`]
/// to render as [`std::fmt::LowerHex`].
#[derive(PartialEq, Debug)]
pub struct HashDisplay(Hash);
impl std::fmt::Display for HashDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:x}", byte)?;
        }
        Ok(())
    }
}

impl From<Hash> for HashDisplay {
    fn from(value: Hash) -> Self {
        HashDisplay(value)
    }
}

/// Trait responsible for encoding data into/from bytes, and producing
/// a [`Hash`] that can be used to uniquely reference it.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Encoder: Clone + ConditionalSync {
    /// Encode a serializable item into its referencable [`Hash`] and its bytes.
    fn encode(&self, item: impl Serialize) -> Result<(Hash, Vec<u8>)>;

    /// Decode bytes into `T`.
    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
}

/// An [`Encoder`] implementation using [`bincode`].
#[cfg(feature = "bincode")]
#[derive(Clone, Default)]
pub struct BincodeEncoder {}

#[cfg(feature = "bincode")]
#[async_trait]
impl Encoder for BincodeEncoder {
    fn encode(&self, item: impl Serialize) -> Result<(Hash, Vec<u8>)> {
        let bytes = bincode::serialize(&item).map_err(|e| Error::Encoding(e.to_string()))?;
        let hash = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&bytes)).to_vec();
        Ok((hash, bytes))
    }

    fn decode<T: DeserializeOwned>(&self, bytes: &HashRef) -> Result<T> {
        bincode::deserialize(&bytes).map_err(|e| Error::Encoding(e.to_string()))
    }
}
