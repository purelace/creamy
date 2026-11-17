use crate::utils::strpool::StringId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagsSymbol {
    pub ident: StringId,
    pub options: Box<[StringId]>,
}
