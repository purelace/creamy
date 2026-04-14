use binrw::binrw;

use crate::{model::Layout, table::TypeTable};

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
