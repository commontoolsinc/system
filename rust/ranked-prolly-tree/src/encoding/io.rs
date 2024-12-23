//! IO utilities to implement [`crate::BasicEncoder`] functionality
//! or to create a new encoder.

use crate::{Block, Error, Result};
use std::{
    cell::Cell,
    io::{Cursor, Write},
};

macro_rules! read_type {
    ( $struct:ident, $fn_name:ident, $ty:ty, $size:expr ) => {
        impl<'a> $struct<'a> {
            #[doc = "Read a `"]
            #[doc = stringify!($ty)]
            #[doc = "` from the reader."]
            pub fn $fn_name(&self) -> Result<$ty> {
                const SIZE: usize = $size;
                let (index, next) = self.check_indices(SIZE)?;
                let mut buff = [0u8; SIZE];
                buff.copy_from_slice(&self.bytes[index..next]);
                let out = <$ty>::from_le_bytes(buff);
                self.index.set(next);
                Ok(out)
            }
        }
    };
}

/// Byte writer.
pub struct Writer {
    cursor: Cursor<Vec<u8>>,
}

impl Writer {
    /// Create a new [`Writer`].
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(vec![]),
        }
    }

    /// Write a `u8` into the writer.
    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        let _ = self.cursor.write(&[value])?;
        Ok(())
    }

    /// Write a `u16` into the writer.
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let _ = self.cursor.write(&value.to_le_bytes())?;
        Ok(())
    }

    /// Write a `u32` into the writer.
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let _ = self.cursor.write(&value.to_le_bytes())?;
        Ok(())
    }

    /// Write a `u64` into the writer.
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        let _ = self.cursor.write(&value.to_le_bytes())?;
        Ok(())
    }

    /// Write bytes into the writer.
    pub fn write_bytes(&mut self, value: &[u8]) -> Result<()> {
        let _ = self.cursor.write(value)?;
        Ok(())
    }

    /// Write a type implementing [`WriteInto`] into the writer.
    pub fn write<W: WriteInto>(&mut self, target: &W) -> Result<()> {
        target.write_into(self)
    }

    /// Convert this writer into the bytes that were written.
    pub fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

/// Types implementing [`WriteInto`] define how they
/// are written via a [`Writer`].
pub trait WriteInto {
    /// Write this struct into a [`Writer`].
    fn write_into(&self, writer: &mut Writer) -> Result<()>;
}

impl WriteInto for &[u8] {
    fn write_into(&self, writer: &mut Writer) -> Result<()> {
        writer.write_u32(u32::try_from(self.len())?)?;
        writer.write_bytes(self)?;
        Ok(())
    }
}

impl<T> WriteInto for Vec<T>
where
    T: WriteInto,
{
    fn write_into(&self, writer: &mut Writer) -> Result<()> {
        writer.write_u32(u32::try_from(self.len())?)?;
        for item in self.iter() {
            writer.write(item)?;
        }
        Ok(())
    }
}

/// Read bytes as references from a source byte slice.
pub struct Reader<'a> {
    bytes: &'a [u8],
    bytes_len: usize,
    index: Cell<usize>,
}

impl<'a> Reader<'a> {
    /// Create a new [`Reader`].
    pub fn new(bytes: &'a [u8]) -> Self {
        Reader {
            bytes,
            bytes_len: bytes.len(),
            index: 0.into(),
        }
    }

    /// Read a `u8` from the reader.
    pub fn read_u8(&self) -> Result<u8> {
        let (index, next) = self.check_indices(1)?;
        self.index.set(next);
        Ok(self.bytes[index])
    }

    /// Read a sequence of `count` bytes from the reader.
    pub fn read_bytes(&self, count: usize) -> Result<&[u8]> {
        let (index, next) = self.check_indices(count)?;
        let out = &self.bytes[index..next];
        self.index.set(next);
        Ok(out)
    }

    /// Read `R` from the reader.
    pub fn read<R: ReadFrom<'a>>(&'a self) -> Result<R> {
        R::read_from(self)
    }

    /// Skip forward `count` bytes.
    pub fn skip(&self, count: usize) -> Result<()> {
        let (_, next) = self.check_indices(count)?;
        self.index.set(next);
        Ok(())
    }

    fn check_indices(&self, size: usize) -> Result<(usize, usize)> {
        let index = self.index.get();
        let next = index + size;
        if next > self.bytes_len {
            return Err(Error::OutOfRange);
        }
        Ok((index, next))
    }
}

read_type!(Reader, read_u16, u16, 2);
read_type!(Reader, read_u32, u32, 4);
read_type!(Reader, read_u64, u64, 8);

/// Types implementing [`ReadFrom`] define how they
/// can be instantiated from a [`Reader`].
pub trait ReadFrom<'a>: Sized {
    /// Instantiate `Self` from a [`Reader`].
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a;
}

impl<'a> ReadFrom<'a> for &'a [u8] {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a,
    {
        let length = reader.read_u32()?;
        reader.read_bytes(length.try_into()?)
    }
}

impl<'a> ReadFrom<'a> for Vec<u8> {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a,
    {
        Ok(<&'a [u8] as ReadFrom<'a>>::read_from(reader)?.to_owned())
    }
}

impl<'a, T> ReadFrom<'a> for Vec<T>
where
    T: ReadFrom<'a>,
{
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a,
    {
        let length = reader.read_u32()?;
        let mut collection = vec![];
        for _ in 0..length {
            collection.push(T::read_from(reader)?);
        }
        Ok(collection)
    }
}

/// Serializable enum distinguishing the block types.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BlockType {
    /// A branch type.
    Branch = 0,
    /// A segment type.
    Segment = 1,
}

impl From<BlockType> for u8 {
    fn from(value: BlockType) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for BlockType {
    type Error = Error;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockType::Branch),
            1 => Ok(BlockType::Segment),
            _ => Err(Error::OutOfRange),
        }
    }
}

impl WriteInto for BlockType {
    fn write_into(&self, writer: &mut Writer) -> Result<()> {
        writer.write_u8(u8::from(*self))
    }
}

impl<'a> ReadFrom<'a> for BlockType {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<BlockType>
    where
        'r: 'a,
    {
        reader.read_u8()?.try_into()
    }
}

impl<K, V> From<&Block<K, V>> for BlockType {
    fn from(value: &Block<K, V>) -> Self {
        match value {
            Block::Branch(_) => BlockType::Branch,
            Block::Segment(_) => BlockType::Segment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_writes_bytes() -> Result<()> {
        let mut writer = Writer::new();
        writer.write_u8(2)?;
        writer.write_u16(11111)?;
        writer.write_u32(222222)?;
        writer.write_u64(3333333)?;
        writer.write_bytes(&[1, 2, 3, 4])?;

        let inner = writer.into_inner();
        let reader = Reader::new(&inner);
        assert_eq!(reader.read_u8()?, 2);
        assert_eq!(reader.read_u16()?, 11111);
        assert_eq!(reader.read_u32()?, 222222);
        assert_eq!(reader.read_u64()?, 3333333);
        assert_eq!(reader.read_bytes(4)?, &[1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn it_checks_bounds_on_read() -> Result<()> {
        let inner = [1u8; 1];
        let reader = Reader::new(&inner);
        assert!(reader.read_u64().is_err());
        assert!(reader.read_u32().is_err());
        assert!(reader.read_u16().is_err());
        reader.read_bytes(1)?;
        assert!(reader.read_u8().is_err());
        assert!(reader.read_bytes(1).is_err());
        Ok(())
    }

    #[test]
    fn it_reads_writes_bytes_with_len() -> Result<()> {
        let mut writer = Writer::new();
        let chunk1 = [1u8, 2, 3];
        let chunk2 = [10u8, 11];
        let chunk3 = [100u8, 2, 9, 8];
        writer.write(&&chunk1[..])?;
        writer.write(&&chunk2[..])?;
        writer.write(&&chunk3[..])?;

        let inner = writer.into_inner();
        let reader = Reader::new(&inner);
        assert_eq!(reader.read::<&[u8]>()?, &chunk1);
        assert_eq!(reader.read::<&[u8]>()?, &chunk2);
        assert_eq!(reader.read::<&[u8]>()?, &chunk3);
        Ok(())
    }

    #[test]
    fn it_reads_writes_collections() -> Result<()> {
        #[derive(Debug, PartialEq)]
        struct Foo(u8);
        impl<'a> ReadFrom<'a> for Foo {
            fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
            where
                'r: 'a,
            {
                Ok(Foo(reader.read_u8()?))
            }
        }
        impl WriteInto for Foo {
            fn write_into(&self, writer: &mut Writer) -> Result<()> {
                writer.write_u8(self.0)
            }
        }

        let mut writer = Writer::new();
        let foo = vec![Foo(1), Foo(2), Foo(3)];
        writer.write(&foo)?;

        let inner = writer.into_inner();
        let reader = Reader::new(&inner);
        assert_eq!(reader.read::<Vec<Foo>>()?, foo);
        Ok(())
    }
}
