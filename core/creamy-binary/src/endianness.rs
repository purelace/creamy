#![cfg_attr(coverage_nightly, coverage(off))]

use crate::DynEndianness;

pub trait Endianness: sealed::Sealed {
    const EMPTY_U8: [u8; size_of::<u8>()] = [0; size_of::<u8>()];
    const EMPTY_U16: [u8; size_of::<u16>()] = [0; size_of::<u16>()];
    const EMPTY_U32: [u8; size_of::<u32>()] = [0; size_of::<u32>()];
    const EMPTY_U64: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
    const EMPTY_U128: [u8; size_of::<u128>()] = [0; size_of::<u128>()];

    const EMPTY_I8: [u8; size_of::<i8>()] = [0; size_of::<i8>()];
    const EMPTY_I16: [u8; size_of::<i16>()] = [0; size_of::<i16>()];
    const EMPTY_I32: [u8; size_of::<i32>()] = [0; size_of::<i32>()];
    const EMPTY_I64: [u8; size_of::<i64>()] = [0; size_of::<i64>()];
    const EMPTY_I128: [u8; size_of::<i128>()] = [0; size_of::<i128>()];

    const EMPTY_F32: [u8; size_of::<f32>()] = [0; size_of::<f32>()];
    const EMPTY_F64: [u8; size_of::<f64>()] = [0; size_of::<f64>()];

    fn write_u8(value: u8) -> [u8; size_of::<u8>()];
    fn write_u16(value: u16) -> [u8; size_of::<u16>()];
    fn write_u32(value: u32) -> [u8; size_of::<u32>()];
    fn write_u64(value: u64) -> [u8; size_of::<u64>()];
    fn write_u128(value: u128) -> [u8; size_of::<u128>()];

    fn write_i8(value: i8) -> [u8; size_of::<i8>()];
    fn write_i16(value: i16) -> [u8; size_of::<i16>()];
    fn write_i32(value: i32) -> [u8; size_of::<i32>()];
    fn write_i64(value: i64) -> [u8; size_of::<i64>()];
    fn write_i128(value: i128) -> [u8; size_of::<i128>()];

    fn write_f32(value: f32) -> [u8; size_of::<f32>()];
    fn write_f64(value: f64) -> [u8; size_of::<f64>()];

    fn read_u8(bytes: [u8; size_of::<u8>()]) -> u8;
    fn read_u16(bytes: [u8; size_of::<u16>()]) -> u16;
    fn read_u32(bytes: [u8; size_of::<u32>()]) -> u32;
    fn read_u64(bytes: [u8; size_of::<u64>()]) -> u64;
    fn read_u128(bytes: [u8; size_of::<u128>()]) -> u128;

    fn read_i8(bytes: [u8; size_of::<i8>()]) -> i8;
    fn read_i16(bytes: [u8; size_of::<i16>()]) -> i16;
    fn read_i32(bytes: [u8; size_of::<i32>()]) -> i32;
    fn read_i64(bytes: [u8; size_of::<i64>()]) -> i64;
    fn read_i128(bytes: [u8; size_of::<i128>()]) -> i128;

    fn read_f32(bytes: [u8; size_of::<f32>()]) -> f32;
    fn read_f64(bytes: [u8; size_of::<f64>()]) -> f64;

    fn as_dyn_endian() -> DynEndianness;
}

pub struct LittleEndian;
pub struct BigEndian;
pub struct NativeEndian;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::LittleEndian {}
    impl Sealed for super::BigEndian {}
    impl Sealed for super::NativeEndian {}
}

macro_rules! impl_endianness_struct {
    ($struct_name:ident, $suffix:ident, $dyn:ident) => {
        impl Endianness for $struct_name {
            paste::item! {
                #[inline(always)] fn write_u8(value: u8) -> [u8; size_of::<u8>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_u16(value: u16) -> [u8; size_of::<u16>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_u32(value: u32) -> [u8; size_of::<u32>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_u64(value: u64) -> [u8; size_of::<u64>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_u128(value: u128) -> [u8; size_of::<u128>()] { value.[<to_ $suffix _bytes>]() }

                #[inline(always)] fn write_i8(value: i8) -> [u8; size_of::<i8>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_i16(value: i16) -> [u8; size_of::<i16>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_i32(value: i32) -> [u8; size_of::<i32>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_i64(value: i64) -> [u8; size_of::<i64>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_i128(value: i128) -> [u8; size_of::<i128>()] { value.[<to_ $suffix _bytes>]() }

                #[inline(always)] fn write_f32(value: f32) -> [u8; size_of::<f32>()] { value.[<to_ $suffix _bytes>]() }
                #[inline(always)] fn write_f64(value: f64) -> [u8; size_of::<f64>()] { value.[<to_ $suffix _bytes>]() }

                #[inline(always)] fn read_u8(bytes: [u8; size_of::<u8>()]) -> u8 { u8::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_u16(bytes: [u8; size_of::<u16>()]) -> u16 { u16::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_u32(bytes: [u8; size_of::<u32>()]) -> u32 { u32::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_u64(bytes: [u8; size_of::<u64>()]) -> u64 { u64::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_u128(bytes: [u8; size_of::<u128>()]) -> u128 { u128::[<from_ $suffix _bytes>](bytes) }

                #[inline(always)] fn read_i8(bytes: [u8; size_of::<i8>()]) -> i8 { i8::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_i16(bytes: [u8; size_of::<i16>()]) -> i16 { i16::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_i32(bytes: [u8; size_of::<i32>()]) -> i32 { i32::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_i64(bytes: [u8; size_of::<i64>()]) -> i64 { i64::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_i128(bytes: [u8; size_of::<i128>()]) -> i128 { i128::[<from_ $suffix _bytes>](bytes) }

                #[inline(always)] fn read_f32(bytes: [u8; size_of::<f32>()]) -> f32 { f32::[<from_ $suffix _bytes>](bytes) }
                #[inline(always)] fn read_f64(bytes: [u8; size_of::<f64>()]) -> f64 { f64::[<from_ $suffix _bytes>](bytes) }

                fn as_dyn_endian() -> DynEndianness {
                    DynEndianness::$dyn
                }
            }
        }
    };
}

impl_endianness_struct!(LittleEndian, le, Little);
impl_endianness_struct!(BigEndian, be, Big);
impl_endianness_struct!(NativeEndian, ne, Native);
