#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::ArraySymbol;
use crate::{constraints::MAX_FIELDS, storage::SymbolKey, table::TypeId, utils::FieldsRange};

pub trait Field {
    fn type_id(&self) -> TypeId;
    fn size(&self) -> u8;
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FieldType {
    Type(TypeId) = 0,
    Array(ArraySymbol) = 1,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FIELDS, FieldsRange)]
#[key(SymbolKey::Field)]
pub struct FieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: FieldType,
}

impl Field for FieldSymbol {
    fn type_id(&self) -> TypeId {
        self.type_id()
    }

    fn size(&self) -> u8 {
        match self.kind() {
            FieldType::Type(sym) => sym.size().value(),
            FieldType::Array(array) => array.total_size(),
        }
    }
}

impl FieldSymbol {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(sym) => sym,
            FieldType::Array(sym) => sym.kind(),
        }
    }
}
