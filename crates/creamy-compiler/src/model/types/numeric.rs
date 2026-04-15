use binrw::binrw;
use compiler_utils::strpool::StringId;

use crate::{
    model::{Layout, ResolvedType},
    table::TypeTable,
};

pub const U8_ID: StringId = StringId::new(0);
pub const U16_ID: StringId = StringId::new(1);
pub const U32_ID: StringId = StringId::new(2);
pub const U64_ID: StringId = StringId::new(3);
pub const U128_ID: StringId = StringId::new(4);

pub const I8_ID: StringId = StringId::new(5);
pub const I16_ID: StringId = StringId::new(6);
pub const I32_ID: StringId = StringId::new(7);
pub const I64_ID: StringId = StringId::new(8);
pub const I128_ID: StringId = StringId::new(9);

pub const F32_ID: StringId = StringId::new(10);
pub const F64_ID: StringId = StringId::new(11);

#[binrw]
#[derive(Debug, Clone, Copy)]
pub enum NumericType {
    #[brw(magic(0u8))]
    U8,
    #[brw(magic(1u8))]
    U16,
    #[brw(magic(2u8))]
    U32,
    #[brw(magic(3u8))]
    U64,
    #[brw(magic(4u8))]
    U128,
    #[brw(magic(5u8))]
    I8,
    #[brw(magic(6u8))]
    I16,
    #[brw(magic(7u8))]
    I32,
    #[brw(magic(8u8))]
    I64,
    #[brw(magic(9u8))]
    I128,
    #[brw(magic(10u8))]
    F32,
    #[brw(magic(11u8))]
    F64,
}

impl Layout for NumericType {
    fn size_of(&self, _: &TypeTable) -> usize {
        match self {
            NumericType::U8 => 1,
            NumericType::U16 => 2,
            NumericType::U32 => 4,
            NumericType::U64 => 8,
            NumericType::U128 => 16,
            NumericType::I8 => 1,
            NumericType::I16 => 2,
            NumericType::I32 => 4,
            NumericType::I64 => 8,
            NumericType::I128 => 16,
            NumericType::F32 => 4,
            NumericType::F64 => 8,
        }
    }

    fn align_of(&self, tt: &TypeTable) -> usize {
        self.size_of(tt)
    }
}

impl ResolvedType for NumericType {
    fn name(&self) -> StringId {
        match self {
            NumericType::U8 => U8_ID,
            NumericType::U16 => U16_ID,
            NumericType::U32 => U32_ID,
            NumericType::U64 => U64_ID,
            NumericType::U128 => U128_ID,
            NumericType::I8 => I8_ID,
            NumericType::I16 => I16_ID,
            NumericType::I32 => I32_ID,
            NumericType::I64 => I64_ID,
            NumericType::I128 => I128_ID,
            NumericType::F32 => F32_ID,
            NumericType::F64 => F64_ID,
        }
    }
}
