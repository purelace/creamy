use alloc::vec::Vec;

use crate::{BinaryReader, BinaryWriter};

impl<T: BinaryReader> BinaryReader for Vec<T> {
    fn read<R: embedded_io::Read, E: crate::Endianness>(
        reader: &mut R,
    ) -> Result<Self, crate::ReadError<R::Error>> {
        let len = u32::read::<R, E>(reader)?;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let item = T::read::<R, E>(reader)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl<T: BinaryWriter> BinaryWriter for Vec<T> {
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        self.as_slice().write::<W, E>(writer)
    }
}
