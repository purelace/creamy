#![cfg_attr(coverage_nightly, coverage(off))]

mod enumeration;
mod field;
mod message;
mod numeric;
mod repr;

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;
pub use enumeration::{EnumSymbol, Number, VariantSymbol, VariantValue};
pub use field::{FieldSymbol, FieldType, SpecialType};
pub use message::*;
pub use numeric::*;
pub use repr::PrimitiveRepr;

use super::storage::SymbolKey;
use crate::{
    constraints::{MAX_BITSET_VALUES, MAX_BITSETS, MAX_FLAGS, MAX_OPTIONS, MAX_STRUCTS},
    table::TypeId,
    utils::{
        BitsetValuesRange, BitsetsRange, FieldsRange, FlagsRange, OptionsRange, Size, StructsRange,
        TypesRange,
    },
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type {
    Numeric(NumericSymbol) = 0,
    Array(ArraySymbol) = 1,
    Flags(FlagsSymbol) = 2,
    Enum(EnumSymbol) = 3,
    Bitset(BitsetSymbol) = 4,
    Struct(StructSymbol) = 5,
    Object(ObjectSymbol) = 6,
}

impl Type {
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            Type::Numeric(s) => s.name(),
            Type::Array(_) => unreachable!(),
            Type::Flags(s) => *s.ident(),
            Type::Enum(s) => *s.ident(),
            Type::Bitset(s) => *s.ident(),
            Type::Struct(s) => *s.ident(),
            Type::Object(s) => *s.ident(),
        }
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArraySymbol {
    kind: TypeId,
    len: Size,
}

impl ArraySymbol {
    #[must_use]
    pub const fn new(kind: TypeId, len: Size) -> Self {
        Self { kind, len }
    }

    #[must_use]
    pub const fn kind(&self) -> TypeId {
        self.kind
    }

    #[must_use]
    pub const fn len(&self) -> Size {
        self.len
    }

    #[must_use]
    pub fn total_size(&self) -> u64 {
        let kind_size = self.kind.size().value();
        let array_len = self.len.value();
        kind_size * array_len
    }
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_STRUCTS, StructsRange)]
pub struct ObjectSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_STRUCTS, StructsRange)]
pub struct StructSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalTypesSymbol {
    types: TypesRange,
}

impl GlobalTypesSymbol {
    #[must_use]
    pub const fn new(types: TypesRange) -> Self {
        Self { types }
    }

    #[must_use]
    pub const fn types(&self) -> TypesRange {
        self.types
    }
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FLAGS, FlagsRange)]
pub struct FlagsSymbol {
    #[token(ident)]
    ident: StringId,
    values: OptionsRange,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_BITSETS, BitsetsRange)]
pub struct BitsetSymbol {
    #[token(ident)]
    ident: StringId,
    values: BitsetValuesRange,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_OPTIONS, OptionsRange)]
#[key(SymbolKey::Option)]
pub struct OptionSymbol {
    #[token(ident)]
    ident: StringId,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_BITSET_VALUES, BitsetValuesRange)]
#[key(SymbolKey::BitsetValue)]
pub struct BitsetValueSymbol {
    #[token(ident)]
    ident: StringId,
    repr: TypeId,
    bits: u8,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayFieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: TypeId,
    len: StringId,
}
