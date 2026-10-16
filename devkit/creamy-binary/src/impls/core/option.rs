use embedded_io::{Read, Write};

use crate::{BinaryReader, BinaryWriter, Endianness, ReadError, WriteError};

impl<T: BinaryWriter> BinaryWriter for Option<T> {
    fn write<W: Write, E: Endianness>(&self, writer: &mut W) -> Result<(), WriteError<W::Error>> {
        if let Some(value) = self {
            true.write::<W, E>(writer)?;
            value.write::<W, E>(writer)?;
        } else {
            false.write::<W, E>(writer)?;
        }

        Ok(())
    }
}

impl<T: BinaryReader> BinaryReader for Option<T> {
    fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
        let is_some = bool::read::<R, E>(reader)?;
        if is_some {
            let value = T::read::<R, E>(reader)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
