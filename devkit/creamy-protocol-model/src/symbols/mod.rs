mod enumeration;
mod field;
mod message;
mod numeric;
mod repr;

use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;
pub use enumeration::{EnumSymbol, VariantSymbol, VariantValue};
pub use field::{FieldSymbol, FieldType};
pub use message::*;
pub use numeric::*;
pub use repr::PrimitiveRepr;

use super::storage::SymbolKey;
use crate::{
    constraints::{
        MAX_BITSET_VALUES, MAX_BITSETS, MAX_FLAGS, MAX_GROUPS, MAX_OPTIONS, MAX_STRUCTS,
    },
    table::TypeId,
    utils::{
        Access, BitsetValuesRange, BitsetsRange, FieldsRange, FlagsRange, GroupsRange,
        MessagesRange, OptionsRange, Size, StructsRange, TypesRange,
    },
};

#[binrw::binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[brw(magic = 0u8)]
    Numeric(NumericSymbol),
    #[brw(magic = 1u8)]
    Array(ArraySymbol),
    #[brw(magic = 2u8)]
    Struct(StructSymbol),
    #[brw(magic = 3u8)]
    Enum(EnumSymbol),
    #[brw(magic = 4u8)]
    Flags(FlagsSymbol),
    #[brw(magic = 5u8)]
    Bitset(BitsetSymbol),
}

impl Type {
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            Type::Numeric(symbol) => symbol.name(),
            Type::Array(_) => unreachable!(),
            Type::Struct(symbol) => symbol.ident(),
            Type::Enum(symbol) => symbol.ident(),
            Type::Flags(symbol) => symbol.ident(),
            Type::Bitset(symbol) => symbol.ident(),
        }
    }

    #[must_use]
    pub const fn with_ident(&self, id: StringId) -> Self {
        match self {
            Type::Numeric(s) => Type::Numeric(*s),
            Type::Array(s) => Type::Array(*s),
            Type::Struct(s) => Type::Struct(s.with_ident(id)),
            Type::Enum(s) => Type::Enum(s.with_ident(id)),
            Type::Flags(s) => Type::Flags(s.with_ident(id)),
            Type::Bitset(s) => Type::Bitset(s.with_ident(id)),
        }
    }
}

#[binrw::binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_STRUCTS, StructsRange)]
pub struct StructSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
}

#[binrw::binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[binrw::binrw]
#[derive(Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_GROUPS, GroupsRange)]
#[key(SymbolKey::Group)]
pub struct GroupSymbol {
    #[token(ident)]
    ident: StringId,
    access: Access,
    messages: MessagesRange,
    types: TypesRange,
}

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FLAGS, FlagsRange)]
pub struct FlagsSymbol {
    #[token(ident)]
    ident: StringId,
    values: OptionsRange,
}

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_BITSETS, BitsetsRange)]
pub struct BitsetSymbol {
    #[token(ident)]
    ident: StringId,
    values: BitsetValuesRange,
}

#[binrw::binrw]
#[derive(Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_OPTIONS, OptionsRange)]
#[key(SymbolKey::Option)]
pub struct OptionSymbol {
    #[token(ident)]
    ident: StringId,
}

#[binrw::binrw]
#[derive(Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_BITSET_VALUES, BitsetValuesRange)]
#[key(SymbolKey::BitsetValue)]
pub struct BitsetValueSymbol {
    #[token(ident)]
    ident: StringId,
    repr: TypeId,
    bits: u8,
}

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayFieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: TypeId,
    len: StringId,
}
