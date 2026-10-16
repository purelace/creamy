use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{Buffer, BufferReader, CustomError, TempBuffer};

impl Buffer for SmolStr {
    type Item = u8;

    type Iter<'a>
        = core::slice::Iter<'a, u8>
    where
        Self: 'a;

    type Length = u32;

    fn length(&self) -> Self::Length {
        self.len() as u32
    }

    fn iterator(&self) -> Self::Iter<'_> {
        self.as_bytes().iter()
    }
}

impl BufferReader for SmolStr {
    type Temp = SmolStrTempBuffer;

    fn from_temp(
        buffer: <<Self as BufferReader>::Temp as TempBuffer>::Type,
    ) -> Result<Self, CustomError> {
        Ok(SmolStr::new(str::from_utf8(&buffer).map_err(|e| {
            CustomError {
                message: "error".into(),
                error: Some(Box::new(e)),
            }
        })?))
    }
}

pub struct SmolStrTempBuffer;
impl TempBuffer for SmolStrTempBuffer {
    type Type = SmallVec<[u8; 22]>;
    type Base = SmolStr;

    fn add(
        buffer: &mut Self::Type,
        index: <<Self as TempBuffer>::Base as Buffer>::Length,
        item: <<Self as TempBuffer>::Base as Buffer>::Item,
    ) {
        buffer[index as usize] = item;
    }

    fn new_with_capacity(capacity: <<Self as TempBuffer>::Base as Buffer>::Length) -> Self::Type {
        smallvec::smallvec![0u8; capacity as usize]
    }
}
