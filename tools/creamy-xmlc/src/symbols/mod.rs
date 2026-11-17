#![cfg_attr(coverage_nightly, coverage(off))]

mod bitset;
mod enumeration;
mod field;
mod flags;
mod message;
mod numeric;
mod repr;

pub use bitset::*;
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
            Type::Flags(s) => s.ident(),
            Type::Enum(s) => s.ident(),
            Type::Bitset(s) => s.ident(),
            Type::Struct(s) => s.ident(),
            Type::Object(s) => s.ident(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArraySymbol {
    pub(crate) kind: TypeId,
    pub(crate) len: Size,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectSymbol {
    pub(crate) ident: StringId,
    pub(crate) fields: Box<[FieldSymbol]>,
}

impl ObjectSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn fields(&self) -> &[FieldSymbol] {
        &self.fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructSymbol {
    pub(crate) ident: StringId,
    pub(crate) fields: Box<[FieldSymbol]>,
}

impl StructSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn fields(&self) -> &[FieldSymbol] {
        &self.fields
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayFieldSymbol {
    pub(crate) ident: StringId,
    pub(crate) kind: TypeId,
    pub(crate) len: StringId,
}
