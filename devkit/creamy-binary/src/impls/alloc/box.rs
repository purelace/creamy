use alloc::{boxed::Box, vec::Vec};

use crate::{BinaryReader, BinaryWriter};

impl<T: BinaryWriter> BinaryWriter for Box<[T]> {
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        let slice: &[T] = self;
        slice.write::<W, E>(writer)
    }
}

impl<T: BinaryReader> BinaryReader for Box<[T]> {
    fn read<R: embedded_io::Read, E: crate::Endianness>(
        reader: &mut R,
    ) -> Result<Self, crate::ReadError<R::Error>> {
        let vec = Vec::read::<R, E>(reader)?;
        Ok(vec.into_boxed_slice())
    }
}
