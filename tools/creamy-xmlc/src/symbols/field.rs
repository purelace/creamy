#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_utils::strpool::StringId;

use super::{ArraySymbol, HeapArraySymbol, T_U8_ID};
use crate::table::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialType {
    SubscriberId,
    ProtocolId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    Type(TypeId),
    StaticArray(ArraySymbol),
    HeapArray(HeapArraySymbol),
    Special(SpecialType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldSymbol {
    pub(crate) ident: StringId,
    pub(crate) kind: FieldType,
    pub(crate) is_padding: bool,
}

impl FieldSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn kind(&self) -> FieldType {
        self.kind
    }

    #[must_use]
    pub const fn is_padding(&self) -> bool {
        self.is_padding
    }

    #[must_use]
    pub fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(s) => s,
            FieldType::StaticArray(s) => s.kind,
            FieldType::HeapArray(s) => s.kind(),
            FieldType::Special(_) => T_U8_ID,
        }
    }

    #[must_use]
    pub fn align(&self) -> u64 {
        match self.kind {
            FieldType::Type(s) => s.align().size(),
            FieldType::StaticArray(s) => s.kind.align().size(),
            FieldType::HeapArray(s) => s.align(),
            FieldType::Special(_) => 1,
        }
    }

    #[must_use]
    pub fn size(&self) -> u64 {
        match self.kind {
            FieldType::Type(s) => s.size().value(),
            FieldType::StaticArray(s) => s.total_size(),
            FieldType::HeapArray(s) => s.size(),
            FieldType::Special(_) => 1,
        }
    }
}
