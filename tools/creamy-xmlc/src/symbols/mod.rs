#![cfg_attr(coverage_nightly, coverage(off))]

mod bitset;
mod enumeration;
mod field;
mod flags;
mod message;
mod numeric;
mod repr;

pub use bitset::*;
use creamy_binary::BinWrite;
pub use enumeration::{EnumSymbol, Number, VariantSymbol};
pub use field::{FieldSymbol, FieldType, SpecialType};
pub use flags::*;
pub use message::*;
pub use numeric::*;
pub use repr::PrimitiveRepr;

use crate::{
    table::TypeId,
    utils::{Size, strpool::StringId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Numeric(NumericSymbol),
    Flags(FlagsSymbol),
    Enum(EnumSymbol),
    Bitset(BitsetSymbol),
    Struct(StructSymbol),
    Object(ObjectSymbol),
}

impl Type {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            Type::Numeric(s) => s.name(),
            Type::Flags(s) => s.ident,
            Type::Enum(s) => s.ident,
            Type::Bitset(s) => s.ident,
            Type::Struct(s) => s.ident,
            Type::Object(s) => s.ident,
        }
    }
}

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArraySymbol {
    pub kind: TypeId,
    pub len: Size,
}

impl ArraySymbol {
    #[must_use]
    pub fn total_size(&self) -> u64 {
        let kind_size = self.kind.size().value();
        let array_len = self.len.value();
        kind_size * array_len
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectSymbol {
    pub ident: StringId,
    pub fields: Box<[FieldSymbol]>,
    pub builtin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructSymbol {
    pub ident: StringId,
    pub fields: Box<[FieldSymbol]>,
    pub builtin: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayFieldSymbol {
    pub ident: StringId,
    pub kind: TypeId,
    pub len: StringId,
}
