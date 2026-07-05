#![cfg_attr(coverage_nightly, coverage(off))]
use std::num::NonZeroU8;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;
use strum::EnumCount;

use crate::{
    table::{TypeId, TypeMeta},
    utils::{Align, Size},
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

pub const BUILTIN_GROUP: NonZeroU8 = NonZeroU8::new(1).unwrap();
pub const U8_META: TypeMeta = TypeMeta::new_with_assert(Size::B1, Align::B1);
pub const U16_META: TypeMeta = TypeMeta::new_with_assert(Size::B2, Align::B2);
pub const U32_META: TypeMeta = TypeMeta::new_with_assert(Size::B4, Align::B4);
pub const U64_META: TypeMeta = TypeMeta::new_with_assert(Size::B8, Align::B8);
pub const U128_META: TypeMeta = TypeMeta::new_with_assert(Size::B16, Align::B8);

pub const T_U8_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(0), U8_META);
pub const T_U16_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(1), U16_META);
pub const T_U32_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(2), U32_META);
pub const T_U64_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(3), U64_META);
pub const T_U128_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(4), U128_META);

pub const T_I8_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(5), U8_META);
pub const T_I16_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(6), U16_META);
pub const T_I32_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(7), U32_META);
pub const T_I64_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(8), U64_META);
pub const T_I128_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(9), U128_META);

pub const T_F32_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(10), U32_META);
pub const T_F64_ID: TypeId = TypeId::new(BUILTIN_GROUP, StringId::new(11), U64_META);

#[derive(EnumCount, BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericSymbol {
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

impl NumericSymbol {
    #[must_use]
    pub const fn name(&self) -> StringId {
        match self {
            NumericSymbol::U8 => U8_ID,
            NumericSymbol::U16 => U16_ID,
            NumericSymbol::U32 => U32_ID,
            NumericSymbol::U64 => U64_ID,
            NumericSymbol::U128 => U128_ID,
            NumericSymbol::I8 => I8_ID,
            NumericSymbol::I16 => I16_ID,
            NumericSymbol::I32 => I32_ID,
            NumericSymbol::I64 => I64_ID,
            NumericSymbol::I128 => I128_ID,
            NumericSymbol::F32 => F32_ID,
            NumericSymbol::F64 => F64_ID,
        }
    }

    #[must_use]
    pub const fn size(&self) -> u8 {
        match self {
            NumericSymbol::U8 | NumericSymbol::I8 => 1,
            NumericSymbol::U16 | NumericSymbol::I16 => 2,
            NumericSymbol::U32 | NumericSymbol::I32 | NumericSymbol::F32 => 4,
            NumericSymbol::U64 | NumericSymbol::I64 | NumericSymbol::F64 => 8,
            NumericSymbol::U128 | NumericSymbol::I128 => 16,
        }
    }

    #[must_use]
    pub const fn align(&self) -> u8 {
        match self {
            NumericSymbol::U8 | NumericSymbol::I8 => 1,
            NumericSymbol::U16 | NumericSymbol::I16 => 2,
            NumericSymbol::U32 | NumericSymbol::I32 | NumericSymbol::F32 => 4,
            NumericSymbol::U64 | NumericSymbol::I64 | NumericSymbol::F64 => 8,
            NumericSymbol::U128 | NumericSymbol::I128 => 16,
        }
    }
}
