//! # Columnar Encoding
//!
//! Each block represents a block type, 1 more more entries with each key
//! comprised of three 32 byte comonents (entity, namespace, attribute)
//! and some value as bytes.
//!
//! The three key components and value for each entry are encoded
//! into a dictionary, once per unique buffer. Each entry is encoded
//! as an index into the dictionary for its components.
//!
//! Non-variable length sections are encoded as little endian.
//!
//! * `version` (u8): Version number.
//! * `block_type` (u8): Either a branch type (0), or segment type (1).
//! * `header_length` (u16): Length in bytes of the `header` section.
//! * `headers` (*): 1 or more bytes representing headers (unused).
//! * `chunk_count` (u32): Number of chunks in the dictionary.
//! * CHUNK: repeats `chunk_count` times.
//!   * `chunk_length` (u32): Length in bytes of the chunk.
//!   * `chunk` (*): Chunk payload.
//! * `entry_count` (u32): Number of entries in the dictionary.
//! * ENTRY: repeats `entry_count` times.
//!   * `key_entity` (u32): Chunk index of entry's entity key.
//!   * `key_namespace` (u32): Chunk index of entry's namespace key.
//!   * `key_attribute` (u32): Chunk index of entry's attribute key.
//!   * `value` (u32): Chunk index of entry's value.
//!
//! ```md
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |    version     |  block_type  |          header_length        |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                            headers*                           |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                          chunk_count                          |
//! +-+-+-+-+-C-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |         H                chunk_length                         | \
//! +-+-+-+-+-U-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+  × chunk_count
//! |         N                   chunk*                            | /
//! +-+-+-+-+-K-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                          entry_count                          |
//! +-+-+-+-+-E-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |         N                 key_entity                          | \
//! +-+-+-+-+-T-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ \
//! |         R                key_namespace                        | \
//! +-+-+-+-+-Y-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+  × entry_count
//! |         E                key_attribute                        | /
//! +-+-+-+-+-N-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ /
//! |         T                    value                            | /
//! +-+-+-+-+-R-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```

use super::io::{ReadFrom, Reader, WriteInto, Writer};
use crate::{Error, MappedKey, Result};
use async_trait::async_trait;
use nonempty::NonEmpty;
use ranked_prolly_tree::{Block, Encoder, Entry, Hash, NodeRef};

const VERSION: u8 = 1;
const EMPTY_ERROR: &str = "No entries found.";
const UNSUPPORTED_VERSION: &str = "Version is not 1.";

/// Dictionaries are an indexed collection of byte chunks.
#[derive(Default)]
struct Dictionary<'a> {
    map: Vec<&'a [u8]>,
}

impl<'a> Dictionary<'a> {
    fn push(&mut self, bytes: &'a [u8]) -> Result<u32> {
        for (index, stored) in self.map.iter().enumerate() {
            if bytes == *stored {
                return Ok(index.try_into()?);
            }
        }
        self.map.push(bytes);
        Ok(u32::try_from(self.map.len() - 1)?)
    }

    fn get(&self, index: u32) -> Result<&&'a [u8]> {
        self.map
            .get(usize::try_from(index)?)
            .ok_or_else(|| Error::OutOfRange)
    }

    fn resolve(
        &self,
        entry: (u32, u32, u32, u32),
    ) -> Result<(&&'a [u8], &&'a [u8], &&'a [u8], &&'a [u8])> {
        Ok((
            self.get(entry.0)?,
            self.get(entry.1)?,
            self.get(entry.2)?,
            self.get(entry.3)?,
        ))
    }
}

impl<'a> WriteInto for Dictionary<'a> {
    fn write_into(self, writer: &mut Writer) -> Result<()> {
        writer.write_u32(self.map.len().try_into()?)?;
        for chunk in self.map {
            writer.write_u32(chunk.len().try_into()?)?;
            writer.write_bytes(&chunk)?;
        }
        Ok(())
    }
}

impl<'a> ReadFrom<'a> for Dictionary<'a> {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Dictionary<'a>>
    where
        'r: 'a,
    {
        let mut dict = Dictionary::default();
        let length = reader.read_u32()?;
        for _ in 0..length {
            let chunk_length = reader.read_u32()?;
            let chunk = reader.read_bytes(chunk_length.try_into()?)?;
            dict.push(chunk)?;
        }
        Ok(dict)
    }
}

/// Entries are an indexed collection of block children
/// as references to their payloads in a [`Dictionary`].
#[derive(Default)]
struct Entries {
    entries: Vec<(u32, u32, u32, u32)>,
}

impl Entries {
    fn push<'a>(
        &mut self,
        dict: &mut Dictionary<'a>,
        key: &'a MappedKey,
        value: &'a [u8],
    ) -> Result<()> {
        let entity = dict.push(key.entity())?;
        let ns = dict.push(key.ns())?;
        let attr = dict.push(key.attr())?;
        let value = dict.push(value)?;
        self.entries.push((entity, ns, attr, value));
        Ok(())
    }

    fn into_inner(self) -> Vec<(u32, u32, u32, u32)> {
        self.entries
    }
}

impl WriteInto for Entries {
    fn write_into(self, writer: &mut Writer) -> Result<()> {
        writer.write_u32(self.entries.len().try_into()?)?;
        for entry in self.entries {
            writer.write_u32(entry.0)?;
            writer.write_u32(entry.1)?;
            writer.write_u32(entry.2)?;
            writer.write_u32(entry.3)?;
        }
        Ok(())
    }
}

impl<'a> ReadFrom<'a> for Entries {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Entries>
    where
        'r: 'a,
    {
        let mut entries = Entries::default();
        let length = reader.read_u32()?;
        for _ in 0..length {
            entries.entries.push((
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
                reader.read_u32()?,
            ));
        }
        Ok(entries)
    }
}

#[derive(Default)]
struct Headers;

impl WriteInto for Headers {
    fn write_into(self, writer: &mut Writer) -> Result<()> {
        writer.write_u16(0)?;
        Ok(())
    }
}

impl<'a> ReadFrom<'a> for Headers {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Headers>
    where
        'r: 'a,
    {
        let length = reader.read_u16()?;
        reader.skip(length.into())?;
        Ok(Headers {})
    }
}

enum BlockType {
    Branch = 0,
    Segment = 1,
}

impl WriteInto for BlockType {
    fn write_into(self, writer: &mut Writer) -> Result<()> {
        writer.write_u8(self as u8)?;
        Ok(())
    }
}

impl<'a> ReadFrom<'a> for BlockType {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<BlockType>
    where
        'r: 'a,
    {
        match reader.read_u8()? {
            0 => Ok(BlockType::Branch),
            1 => Ok(BlockType::Segment),
            _ => Err(Error::OutOfRange),
        }
    }
}

impl From<&Block<MappedKey>> for BlockType {
    fn from(value: &Block<MappedKey>) -> Self {
        match value {
            Block::Branch(_) => BlockType::Branch,
            Block::Segment(_) => BlockType::Segment,
        }
    }
}

/// A columnar [`Encoder`] implementation.
#[derive(Clone, Default)]
pub struct ColumnarEncoder {}

impl ColumnarEncoder {
    /// Serializes a [`Block`] into encoded bytes.
    pub fn serialize(block: &Block<MappedKey>) -> Result<Vec<u8>> {
        let mut dict = Dictionary::default();
        let mut entries = Entries::default();

        for (key, value) in block.key_values() {
            entries.push(&mut dict, key, value)?;
        }

        let mut writer = Writer::new();

        writer.write(VERSION)?;
        writer.write(BlockType::from(block))?;
        writer.write(Headers {})?;
        writer.write(dict)?;
        writer.write(entries)?;
        Ok(writer.into_inner())
    }

    /// Deserializes encoded bytes into a [`Block`].
    pub fn deserialize(bytes: &[u8]) -> Result<Block<MappedKey>> {
        let reader = Reader::new(bytes);

        if reader.read_u8()? != VERSION {
            return Err(Error::Encoding(UNSUPPORTED_VERSION.into()));
        };
        let block_type = reader.read::<BlockType>()?;
        let _headers = reader.read::<Headers>()?;
        let dictionary = reader.read::<Dictionary>()?;
        let entries = reader.read::<Entries>()?;

        match block_type {
            BlockType::Branch => {
                let mut children = vec![];
                for indices in entries.into_inner() {
                    let (entity, ns, attr, value) = dictionary.resolve(indices)?;
                    let key = MappedKey::from_components(*entity, *ns, *attr);
                    let value = (*value).to_owned();
                    children.push(NodeRef::new(key, value));
                }
                let children = NonEmpty::try_from(children)
                    .map_err(|_| Error::Encoding(EMPTY_ERROR.into()))?;
                Ok(Block::branch(children))
            }
            BlockType::Segment => {
                let mut children = vec![];
                for indices in entries.into_inner() {
                    let (entity, ns, attr, value) = dictionary.resolve(indices)?;
                    let key = MappedKey::from_components(*entity, *ns, *attr);
                    let value = (*value).to_owned();
                    children.push(Entry::new(key, value));
                }
                let children = NonEmpty::try_from(children)
                    .map_err(|_| Error::Encoding(EMPTY_ERROR.into()))?;
                Ok(Block::segment(children))
            }
        }
    }
}

#[async_trait]
impl Encoder<MappedKey> for ColumnarEncoder {
    fn encode(&self, block: &Block<MappedKey>) -> ranked_prolly_tree::Result<(Hash, Vec<u8>)> {
        let bytes = Self::serialize(block)?;
        let hash = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&bytes)).to_vec();
        Ok((hash, bytes))
    }

    fn decode(&self, bytes: &[u8]) -> ranked_prolly_tree::Result<Block<MappedKey>> {
        Ok(Self::deserialize(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ranked_prolly_tree::BincodeEncoder;

    #[test]
    fn columnar_encoding() -> Result<()> {
        let data = [vec![0; 1024], vec![1; 1024], vec![2; 1024]];
        let data = [
            (MappedKey::new("entity1", "ns1", "a"), &data[0]),
            (MappedKey::new("entity1", "ns1", "b"), &data[0]),
            (MappedKey::new("entity1", "ns1", "c"), &data[1]),
            (MappedKey::new("entity1", "ns1", "d"), &data[1]),
            (MappedKey::new("entity1", "ns2", "e"), &data[1]),
            (MappedKey::new("entity1", "ns2", "a"), &data[1]),
            (MappedKey::new("entity2", "2ns", "f"), &data[0]),
            (MappedKey::new("entity2", "2ns", "g"), &data[2]),
            (MappedKey::new("entity2", "ns1", "h"), &data[0]),
        ];

        let children: Vec<Entry<MappedKey>> = data
            .into_iter()
            .map(|row| Entry::new(row.0, row.1.to_owned()))
            .collect();
        let block = Block::segment(children.try_into().unwrap());

        let columnar_decoded = {
            let encoder = ColumnarEncoder::default();
            println!("# Columnar Encoding");
            let (_, encoded) = encoder.encode(&block).unwrap();
            println!("Encoded size: {}", encoded.len());
            let decoded = encoder.decode(&encoded).unwrap();
            assert_eq!(block, decoded);
            decoded
        };
        {
            let encoder = BincodeEncoder::default();
            println!("# Bincode Encoding");
            let (_, encoded) = encoder.encode(&block).unwrap();
            println!("Encoded size: {}", encoded.len());
            let decoded = encoder.decode(&encoded).unwrap();
            assert_eq!(block, decoded);
            assert_eq!(block, columnar_decoded);
        }
        Ok(())
    }
}
