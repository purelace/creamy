use core::mem::MaybeUninit;

use crate::{BinaryReader, BinaryWriter, Buffer, ReadError};

impl<T: BinaryWriter, const N: usize> BinaryWriter for [T; N] {
    fn write<W: embedded_io::Write, E: crate::Endianness>(
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

impl<T> Buffer for &[T] {
    type Item = T;

    type Iter<'a>
        = core::slice::Iter<'a, T>
    where
        Self: 'a;

    type Length = u32;

    fn length(&self) -> Self::Length {
        self.len() as u32
    }

    fn iterator(&self) -> Self::Iter<'_> {
        self.iter()
    }
}
