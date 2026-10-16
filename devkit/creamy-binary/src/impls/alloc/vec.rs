use alloc::{boxed::Box, vec::Vec};
use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{BinaryReader, Buffer, BufferReader, CustomError, TempBuffer};

impl<T> Buffer for Vec<T> {
    type Item = T;

    type Iter<'a>
        = core::slice::Iter<'a, Self::Item>
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

impl<T: BinaryReader> BufferReader for Vec<T> {
    type Temp = Temp<T>;

    fn from_temp(
        buffer: <<Self as BufferReader>::Temp as TempBuffer>::Type,
    ) -> Result<Self, CustomError> {
        Ok(unsafe { buffer.assume_init().into_vec() })
    }
}

pub struct Temp<T: BinaryReader> {
    _phantom: PhantomData<T>,
}

impl<T: BinaryReader> TempBuffer for Temp<T> {
    type Type = Box<[MaybeUninit<T>]>;
    type Base = Vec<T>;

    fn add(
        buffer: &mut Self::Type,
        index: <<Self as TempBuffer>::Base as Buffer>::Length,
        item: <<Self as TempBuffer>::Base as Buffer>::Item,
    ) {
        buffer[index as usize] = MaybeUninit::new(item);
    }

    fn new_with_capacity(capacity: <<Self as TempBuffer>::Base as Buffer>::Length) -> Self::Type {
        Box::new_zeroed_slice(capacity as usize)
    }
}
