use alloc::{boxed::Box, string::String, vec::Vec};

use embedded_io::{Read, Write};

use crate::{BinaryReader, BinaryWriter, CustomError, Endianness, ReadError, WriteError};

impl BinaryWriter for &str {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.as_bytes().write::<W, E>(writer)
    }
}

impl BinaryWriter for char {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        (*self as u32).write::<W, E>(writer)
    }
}

impl BinaryReader for char {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let value = u32::read::<R, E>(reader)?;
        Ok(char::from_u32(value).unwrap())
    }
}

impl BinaryReader for String {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let bytes = Vec::read::<R, E>(reader)?;
        let string = String::from_utf8(bytes).map_err(|e| {
            ReadError::Custom(CustomError {
                message: String::new(),
                error: Some(Box::new(e)),
            })
        })?;
        Ok(string)
    }
}

impl BinaryWriter for String {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        self.as_bytes().write::<W, E>(writer)?;
        Ok(())
    }
}
