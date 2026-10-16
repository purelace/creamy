use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroU128,
};

use embedded_io::{Read, Write};

use crate::{BinaryReader, BinaryWriter, Endianness, ReadError, WriteError};

macro_rules! impl_num_type {
    ($type: ident, $default:ident, $write: ident, $read: ident) => {
        impl BinaryWriter for $type {
            fn write<W: Write, E: Endianness>(
                &self,
                writer: &mut W,
            ) -> Result<(), WriteError<W::Error>> {
                writer.write(&E::$write(*self))?;
                Ok(())
            }
        }

        impl BinaryReader for $type {
            fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
                let mut buf = E::$default;
                let read = reader.read(&mut buf)?;
                if read == 0 {
                    return Err(ReadError::EndOfBuffer);
                }
                Ok(E::$read(buf))
            }
        }
    };
}

impl_num_type!(u8, EMPTY_U8, write_u8, read_u8);
impl_num_type!(u16, EMPTY_U16, write_u16, read_u16);
impl_num_type!(u32, EMPTY_U32, write_u32, read_u32);
impl_num_type!(u64, EMPTY_U64, write_u64, read_u64);
impl_num_type!(u128, EMPTY_U128, write_u128, read_u128);

impl_num_type!(i8, EMPTY_I8, write_i8, read_i8);
impl_num_type!(i16, EMPTY_I16, write_i16, read_i16);
impl_num_type!(i32, EMPTY_I32, write_i32, read_i32);
impl_num_type!(i64, EMPTY_I64, write_i64, read_i64);
impl_num_type!(i128, EMPTY_I128, write_i128, read_i128);

impl_num_type!(f32, EMPTY_F32, write_f32, read_f32);
impl_num_type!(f64, EMPTY_F64, write_f64, read_f64);

macro_rules! impl_non_zero_num_type {
    ($type: ident, $original: ident) => {
        impl BinaryWriter for $type {
            fn write<W: Write, E: Endianness>(
                &self,
                writer: &mut W,
            ) -> Result<(), WriteError<W::Error>> {
                self.get().write::<W, E>(writer)?;
                Ok(())
            }
        }

        impl BinaryReader for $type {
            fn read<R: Read, E: Endianness>(reader: &mut R) -> Result<Self, ReadError<R::Error>> {
                let value = $original::read::<R, E>(reader)?;
                match $type::new(value) {
                    Some(value) => Ok(value),
                    None => Err(ReadError::ZeroValue),
                }
            }
        }
    };
}

impl_non_zero_num_type!(NonZeroU8, u8);
impl_non_zero_num_type!(NonZeroU16, u16);
impl_non_zero_num_type!(NonZeroU32, u32);
impl_non_zero_num_type!(NonZeroU64, u64);
impl_non_zero_num_type!(NonZeroU128, u128);

impl_non_zero_num_type!(NonZeroI8, i8);
impl_non_zero_num_type!(NonZeroI16, i16);
impl_non_zero_num_type!(NonZeroI32, i32);
impl_non_zero_num_type!(NonZeroI64, i64);
impl_non_zero_num_type!(NonZeroI128, i128);
