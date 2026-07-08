use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;

use crate::{
    constraints::MAX_FIELDS, define_readonly_struct, model::symbols::ArraySymbol, table::TypeId,
    utils::FieldsRange,
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    #[brw(magic(0u8))]
    Type(TypeId),
    #[brw(magic(1u8))]
    Array(ArraySymbol),
}

define_readonly_struct! {
    [element(MAX_FIELDS, FieldsRange)]
    struct FieldSymbol {
        name: StringId,
        kind: FieldType,
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
