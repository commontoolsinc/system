use crate::{Error, Result};
use std::{
    cell::Cell,
    io::{Cursor, Write},
};

pub struct Writer {
    cursor: Cursor<Vec<u8>>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::new(vec![]),
        }
    }

    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        let _ = self.cursor.write(&[value])?;
        Ok(())
    }

    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let _ = self.cursor.write(&value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let _ = self.cursor.write(&value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_bytes(&mut self, value: &[u8]) -> Result<()> {
        let _ = self.cursor.write(value)?;
        Ok(())
    }

    pub fn write<W: WriteInto>(&mut self, target: W) -> Result<()> {
        target.write_into(self)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.cursor.into_inner()
    }
}

pub trait WriteInto {
    fn write_into(self, writer: &mut Writer) -> Result<()>;
}

impl WriteInto for u8 {
    fn write_into(self, writer: &mut Writer) -> Result<()> {
        writer.write_u8(self)
    }
}

/// Read bytes as references from a source byte slice.
pub struct Reader<'a> {
    bytes: &'a [u8],
    bytes_len: usize,
    index: Cell<usize>,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Reader {
            bytes,
            bytes_len: bytes.len(),
            index: 0.into(),
        }
    }

    pub fn read_u8(&self) -> Result<u8> {
        let (index, next) = self.check_indices(1)?;
        self.index.set(next);
        Ok(self.bytes[index])
    }

    pub fn read_u16(&self) -> Result<u16> {
        let (index, next) = self.check_indices(2)?;
        let mut buff = [0u8; 2];
        buff.copy_from_slice(&self.bytes[index..next]);
        let out = u16::from_le_bytes(buff);
        self.index.set(next);
        Ok(out)
    }

    pub fn read_u32(&self) -> Result<u32> {
        let (index, next) = self.check_indices(4)?;
        let mut buff = [0u8; 4];
        buff.copy_from_slice(&self.bytes[index..next]);
        let out = u32::from_le_bytes(buff);
        self.index.set(next);
        Ok(out)
    }

    pub fn read_bytes(&self, count: usize) -> Result<&[u8]> {
        let (index, next) = self.check_indices(count)?;
        let out = &self.bytes[index..next];
        self.index.set(next);
        Ok(out)
    }

    pub fn read<R: ReadFrom<'a>>(&'a self) -> Result<R> {
        R::read_from(self)
    }

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

pub trait ReadFrom<'a>: Sized {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a;
}

impl<'a> ReadFrom<'a> for u8 {
    fn read_from<'r>(reader: &'r Reader<'a>) -> Result<Self>
    where
        'r: 'a,
    {
        reader.read_u8()
    }
}
