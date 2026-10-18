use core::mem::MaybeUninit;

use crate::{BinaryReader, BinaryWriter, ReadError};

impl<T: BinaryWriter, const N: usize> BinaryWriter for [T; N] {
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        for value in self {
            value.write::<W, E>(writer)?;
        }
        Ok(())
    }
}

impl<T: BinaryReader, const N: usize> BinaryReader for [T; N] {
    fn read<R: embedded_io::Read, E: crate::Endianness>(
        reader: &mut R,
    ) -> Result<Self, ReadError<R::Error>> {
        let mut buffer: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for cell in &mut buffer {
            let value = T::read::<R, E>(reader)?;
            *cell = MaybeUninit::new(value);
        }

        let buffer = unsafe {
            let ptr = buffer.as_ptr().cast::<[T; N]>();
            //core::mem::forget(buffer);
            core::ptr::read(ptr)
        };

        Ok(buffer)
    }
}

impl<T: BinaryWriter> BinaryWriter for &[T] {
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        (self.len() as u32).write::<W, E>(writer)?;
        for i in *self {
            i.write::<W, E>(writer)?;
        }

        Ok(())
    }
}
