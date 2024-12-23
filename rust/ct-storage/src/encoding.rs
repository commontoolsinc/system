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

use crate::Key;
use async_trait::async_trait;
use nonempty::NonEmpty;
use ranked_prolly_tree::{
    io::{BlockType, ReadFrom, Reader, WriteInto, Writer},
    Block, Encoder, Entry, Error, Hash, NodeRef, Result,
};

const VERSION: u8 = 1;
const UNSUPPORTED_VERSION: &str = "Version is not 1.";
const COMPONENT_LEN: usize = 4;

/// Entries are an indexed collection of block children
/// as references to their payloads in a dictionary.
#[derive(Default)]
struct Entries<'a> {
    dictionary: Vec<&'a [u8]>,
    items: Vec<[u32; COMPONENT_LEN]>,
}

impl<'a> Entries<'a> {
    fn push(&mut self, key: &'a Key, value: &'a [u8]) -> Result<()> {
        fn encode<'a>(dict: &mut Vec<&'a [u8]>, bytes: &'a [u8]) -> Result<u32> {
            for (index, stored) in dict.iter().enumerate() {
                if bytes == *stored {
                    return Ok(index.try_into()?);
                }
            }
            dict.push(bytes);
            Ok(u32::try_from(dict.len() - 1)?)
        }
        let mut entry = [0; COMPONENT_LEN];
        entry[0] = encode(&mut self.dictionary, key.entity())?;
        entry[1] = encode(&mut self.dictionary, key.ns())?;
        entry[2] = encode(&mut self.dictionary, key.attr())?;
        entry[3] = encode(&mut self.dictionary, value)?;
        self.items.push(entry);
        Ok(())
    }

    fn into_block(self, block_type: BlockType) -> Result<Block<Key, Vec<u8>>> {
        fn resolve<'a>(dictionary: &'a Vec<&'a [u8]>, index: u32) -> Result<&'a &'a [u8]> {
            dictionary
                .get(usize::try_from(index)?)
                .ok_or_else(|| Error::OutOfRange)
        }

        let dict = self.dictionary;
        match block_type {
            BlockType::Branch => {
                let mut children = vec![];

                for indices in self.items.into_iter() {
                    let key = Key::from_slices(
                        resolve(&dict, indices[0])?,
                        resolve(&dict, indices[1])?,
                        resolve(&dict, indices[2])?,
                    )?;
                    let value = (*resolve(&dict, indices[3])?).to_owned();
                    children.push(NodeRef::new(key, value));
                }
                let children = NonEmpty::try_from(children).map_err(|_| Error::EmptyChildren)?;
                Ok(Block::branch(children))
            }
            BlockType::Segment => {
                let mut children = vec![];
                for indices in self.items.into_iter() {
                    let key = Key::from_slices(
                        resolve(&dict, indices[0])?,
                        resolve(&dict, indices[1])?,
                        resolve(&dict, indices[2])?,
                    )?;
                    let value = (*resolve(&dict, indices[3])?).to_owned();
                    children.push(Entry::new(key, value));
                }
                let children = NonEmpty::try_from(children).map_err(|_| Error::EmptyChildren)?;
                Ok(Block::segment(children))
            }
        }
    }
}

impl<'a> WriteInto for Entries<'a> {
    fn write_into(&self, writer: &mut Writer) -> Result<()> {
        writer.write(&self.dictionary)?;
        writer.write_u32(self.items.len().try_into()?)?;
        for indices in self.items.iter() {
            for index in indices.iter() {
                writer.write_u32(*index)?;
            }
        }
        Ok(())
    }
}

impl<'a> ReadFrom<'a> for Entries<'a> {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Entries<'a>>
    where
        'r: 'a,
    {
        let dictionary = reader.read::<Vec<&'a [u8]>>()?;
        let mut items: Vec<[u32; COMPONENT_LEN]> = vec![];
        let length = reader.read_u32()?;
        for _ in 0..length {
            let mut item = [0; COMPONENT_LEN];
            for index in 0..COMPONENT_LEN {
                item[index] = reader.read_u32()?;
            }
            items.push(item);
        }
        Ok(Entries { dictionary, items })
    }
}

/// A columnar [`Encoder`] implementation.
#[derive(Clone, Default)]
pub struct ColumnarEncoder {}

impl ColumnarEncoder {
    /// Serializes a [`Block`] into encoded bytes.
    pub fn serialize(block: &Block<Key, Vec<u8>>) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        let mut entries = Entries::default();
        let block_type = BlockType::from(block);
        writer.write_u8(VERSION)?;
        writer.write(&block_type)?;
        writer.write_u16(0)?;

        match block_type {
            BlockType::Branch => {
                let refs = block.node_refs()?;
                for node_ref in refs {
                    entries.push(node_ref.boundary(), node_ref.hash().as_ref())?;
                }
                writer.write(&entries)?;
            }
            BlockType::Segment => {
                let items = block.entries()?;
                for entry in items {
                    entries.push(&entry.key, &entry.value)?;
                }
                writer.write(&entries)?;
            }
        }

        Ok(writer.into_inner())
    }

    /// Deserializes encoded bytes into a [`Block`].
    pub fn deserialize(bytes: &[u8]) -> Result<Block<Key, Vec<u8>>> {
        let reader = Reader::new(bytes);

        if reader.read_u8()? != VERSION {
            return Err(Error::Encoding(UNSUPPORTED_VERSION.into()));
        };
        let block_type = reader.read::<BlockType>()?;
        let header_len = reader.read_u16()?;
        reader.skip(header_len as usize)?;
        let entries = reader.read::<Entries>()?;
        entries.into_block(block_type)
    }
}

#[async_trait]
impl Encoder<Key, Vec<u8>> for ColumnarEncoder {
    fn encode(&self, block: &Block<Key, Vec<u8>>) -> Result<(Hash, Vec<u8>)> {
        let bytes = Self::serialize(block)?;
        let hash = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&bytes)).to_vec();
        Ok((hash, bytes))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Block<Key, Vec<u8>>> {
        Ok(Self::deserialize(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ranked_prolly_tree::BasicEncoder;

    #[test]
    fn columnar_encoding() -> Result<()> {
        let data = [vec![0; 1024], vec![1; 1024], vec![2; 1024]];
        let keys = [
            (Key::new("entity1", "ns1", "a"), &data[0]),
            (Key::new("entity1", "ns1", "b"), &data[0]),
            (Key::new("entity1", "ns1", "c"), &data[1]),
            (Key::new("entity1", "ns1", "d"), &data[1]),
            (Key::new("entity1", "ns2", "e"), &data[1]),
            (Key::new("entity1", "ns2", "a"), &data[1]),
            (Key::new("entity2", "2ns", "f"), &data[0]),
            (Key::new("entity2", "2ns", "g"), &data[2]),
            (Key::new("entity2", "ns1", "h"), &data[0]),
        ];

        let children: Vec<Entry<Key, Vec<u8>>> = keys
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
            let encoder = BasicEncoder::default();
            println!("# Basic Encoding");
            let (_, encoded) = encoder.encode(&block).unwrap();
            println!("Encoded size: {}", encoded.len());
            let decoded = encoder.decode(&encoded).unwrap();
            assert_eq!(block, decoded);
            assert_eq!(block, columnar_decoded);
        }
        Ok(())
    }
}
