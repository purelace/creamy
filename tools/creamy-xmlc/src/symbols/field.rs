#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::BinWrite;

use super::{ArraySymbol, HeapArraySymbol, T_U8_ID};
use crate::{table::TypeId, utils::strpool::StringId};

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SpecialType {
    SubscriberId = 0,
    ProtocolId = 1,
}

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FieldType {
    Type(TypeId) = 0,
    StaticArray(ArraySymbol) = 1,
    HeapArray(HeapArraySymbol) = 2,
    Special(SpecialType) = 3,
}

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldSymbol {
    pub ident: StringId,
    pub kind: FieldType,
    pub is_padding: bool,
}

impl FieldSymbol {
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(s) => s,
            FieldType::StaticArray(s) => s.kind,
            FieldType::HeapArray(s) => s.kind,
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
