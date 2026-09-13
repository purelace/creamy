use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use crate::{
    constraints::MAX_FIELDS,
    model::{storage::SymbolKey, symbols::ArraySymbol},
    table::TypeId,
    utils::FieldsRange,
};

#[binrw::binrw]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    #[brw(magic(0u8))]
    Type(TypeId),
    #[brw(magic(1u8))]
    Array(ArraySymbol),
}

#[binrw::binrw]
#[derive(Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FIELDS, FieldsRange)]
#[key(SymbolKey::Field)]
pub struct FieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: FieldType,
}
crate::define_readonly_struct!(@impl_methods FieldSymbol {
    ident: StringId, kind: FieldType,
});

impl FieldSymbol {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(sym) => sym,
            FieldType::Array(sym) => sym.kind(),
        }
    }
}
