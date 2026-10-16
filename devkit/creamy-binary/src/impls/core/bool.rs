use embedded_io::{Read, Write};

use crate::{BinaryReader, BinaryWriter, Endianness, ReadError, WriteError};

impl BinaryWriter for bool {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        u8::from(*self).write::<W, E>(writer)
    }
}

impl BinaryReader for bool {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        Ok(u8::read::<R, E>(reader)? == 1)
    }
}
