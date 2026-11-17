use core::fmt::Display;

use super::PrimitiveRepr;
use crate::{error::SemanticError, table::TypeMeta, utils::strpool::StringId};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Number {
    Signed(i64),
    Unsigned(u64),
}

impl Display for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Number::Signed(s) => write!(f, "{s}"),
            Number::Unsigned(u) => write!(f, "{u}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantSymbol {
    pub ident: StringId,
    pub value: Number,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumSymbol {
    pub ident: StringId,
    pub repr: PrimitiveRepr,
    pub variants: Box<[VariantSymbol]>,
    pub builtin: bool,
}

impl EnumSymbol {
    /// Returns the [`TypeMeta`] of the [`EnumSymbol`] instance.
    ///
    /// # Errors
    ///
    /// See [`TypeMeta`].
    pub fn meta(&self) -> Result<TypeMeta, SemanticError> {
        let ty = self.repr.as_numberic_symbol();
        TypeMeta::try_new(ty.size(), ty.align())
    }
}
