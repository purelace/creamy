use crate::{table::TypeId, utils::strpool::StringId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitsetValueSymbol {
    pub ident: StringId,
    pub repr: TypeId,
    pub width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitsetSymbol {
    pub ident: StringId,
    pub values: Box<[BitsetValueSymbol]>,
}
