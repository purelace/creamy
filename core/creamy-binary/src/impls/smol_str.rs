use alloc::boxed::Box;

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{BinaryReader, BinaryWriter, CustomError, ReadError};

impl BinaryWriter for SmolStr {
    fn write<W: embedded_io::Write<Error: Send + Sync + 'static>, E: crate::Endianness>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::WriteError<W::Error>> {
        self.as_str().write::<W, E>(writer)
    }
}

impl BinaryReader for SmolStr {
    fn read<R: embedded_io::Read, E: crate::Endianness>(
        reader: &mut R,
    ) -> Result<Self, crate::ReadError<R::Error>> {
        let len = u32::read::<R, E>(reader)?;
        let mut temp = SmallVec::<[u8; 22]>::with_capacity(len as usize);
        for _ in 0..len {
            let byte = u8::read::<R, E>(reader)?;
            temp.push(byte);
        }

        Ok(SmolStr::new(str::from_utf8(&temp).map_err(|e| {
            ReadError::Custom(CustomError {
                message: "error".into(),
                error: Some(Box::new(e)),
            })
        })?))
    }
}
