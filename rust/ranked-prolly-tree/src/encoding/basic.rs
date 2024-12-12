//! # Basic Encoding
//!
//! Each block represents a block type, 1 more more entries with each key
//! comprised of three 32 byte comonents (entity, namespace, attribute)
//! and some value as bytes.

use super::io::{BlockType, Reader, Writer};
use crate::{Block, Encoder, Entry, Error, Hash, Key, NodeRef, Result};
use async_trait::async_trait;
use ct_common::ConditionalSync;
use nonempty::NonEmpty;

const TRY_FROM_BYTES_FAILURE: &str = "Could not read component from bytes.";

/// A basic [`Encoder`] implementation for keys and values
/// that can be represented as bytes.
#[derive(Clone, Default)]
pub struct BasicEncoder {}

impl BasicEncoder {
    /// Serializes a [`Block`] into encoded bytes.
    pub fn serialize<K, V>(block: &Block<K, V>) -> Result<Vec<u8>>
    where
        K: Key + AsRef<[u8]> + 'static,
        V: ConditionalSync + AsRef<[u8]>,
    {
        let mut writer = Writer::new();
        let block_type = BlockType::from(block);
        writer.write(&block_type)?;

        match block_type {
            BlockType::Branch => {
                let refs = block.node_refs()?;
                writer.write_u32(refs.len().try_into()?)?;
                for node_ref in refs {
                    writer.write(&node_ref.boundary().as_ref())?;
                    writer.write::<&[u8]>(&node_ref.hash().as_ref())?;
                }
            }
            BlockType::Segment => {
                let entries = block.entries()?;
                writer.write_u32(entries.len().try_into()?)?;
                for entry in entries {
                    writer.write(&entry.key.as_ref())?;
                    writer.write(&entry.value.as_ref())?;
                }
            }
        }
        Ok(writer.into_inner())
    }

    /// Deserializes encoded bytes into a [`Block`].
    pub fn deserialize<K, V>(bytes: &[u8]) -> Result<Block<K, V>>
    where
        K: Key + TryFrom<Vec<u8>> + 'static,
        V: ConditionalSync + TryFrom<Vec<u8>>,
    {
        let reader = Reader::new(bytes);
        let block_type = reader.read::<BlockType>()?;
        let child_count = reader.read_u32()?;
        match block_type {
            BlockType::Branch => {
                let mut children = vec![];
                for _ in 0..child_count {
                    let boundary = reader
                        .read::<Vec<u8>>()?
                        .try_into()
                        .map_err(|_| Error::Encoding(TRY_FROM_BYTES_FAILURE.into()))?;
                    let hash = reader
                        .read::<Vec<u8>>()?
                        .try_into()
                        .map_err(|_| Error::Encoding(TRY_FROM_BYTES_FAILURE.into()))?;
                    children.push(NodeRef::new(boundary, hash))
                }
                let children = NonEmpty::try_from(children).map_err(|_| Error::EmptyChildren)?;
                Ok(Block::branch(children))
            }
            BlockType::Segment => {
                let mut children = vec![];
                for _ in 0..child_count {
                    let key = reader
                        .read::<Vec<u8>>()?
                        .try_into()
                        .map_err(|_| Error::Encoding(TRY_FROM_BYTES_FAILURE.into()))?;
                    let value = reader
                        .read::<Vec<u8>>()?
                        .try_into()
                        .map_err(|_| Error::Encoding(TRY_FROM_BYTES_FAILURE.into()))?;
                    children.push(Entry::new(key, value))
                }
                let children = NonEmpty::try_from(children).map_err(|_| Error::EmptyChildren)?;
                Ok(Block::segment(children))
            }
        }
    }
}

#[async_trait]
impl<K, V> Encoder<K, V> for BasicEncoder
where
    K: Key + AsRef<[u8]> + TryFrom<Vec<u8>> + 'static,
    V: ConditionalSync + AsRef<[u8]> + TryFrom<Vec<u8>>,
{
    fn encode(&self, block: &Block<K, V>) -> Result<(Hash, Vec<u8>)> {
        let bytes = Self::serialize(block)?;
        let hash = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&bytes)).to_vec();
        Ok((hash, bytes))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Block<K, V>> {
        Ok(Self::deserialize(&bytes)?)
    }
}
