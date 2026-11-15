#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::{ArraySymbol, HeapArraySymbol};
use crate::{constraints::MAX_FIELDS, storage::SymbolKey, table::TypeId, utils::FieldsRange};

pub trait Field {
    fn type_id(&self) -> TypeId;
    fn size(&self) -> u64;
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FieldType {
    Type(TypeId) = 0,
    StaticArray(ArraySymbol) = 1,
    HeapArray(HeapArraySymbol) = 2,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FIELDS, FieldsRange)]
#[key(SymbolKey::Field)]
pub struct FieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: FieldType,
    is_padding: bool,
}

impl Field for FieldSymbol {
    fn type_id(&self) -> TypeId {
        self.type_id()
    }

    fn size(&self) -> u64 {
        match self.kind() {
            FieldType::Type(s) => s.size().value(),
            FieldType::StaticArray(s) => s.total_size(),
            FieldType::HeapArray(_) => {
                const ARRAY_LENGTH_FIELD: u64 = 4;
                ARRAY_LENGTH_FIELD
            }
        }
    }
}

impl FieldSymbol {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(s) => s,
            FieldType::StaticArray(s) => s.kind(),
            FieldType::HeapArray(s) => s.kind(),
        }
    }
}
