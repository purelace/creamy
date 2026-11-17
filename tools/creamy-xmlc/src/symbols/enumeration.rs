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
    pub(crate) ident: StringId,
    pub(crate) value: Number,
}

impl VariantSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn value(&self) -> Number {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumSymbol {
    pub(crate) ident: StringId,
    pub(crate) repr: PrimitiveRepr,
    pub(crate) variants: Box<[VariantSymbol]>,
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

    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn repr(&self) -> PrimitiveRepr {
        self.repr
    }

    #[must_use]
    pub const fn variants(&self) -> &[VariantSymbol] {
        &self.variants
    }
}
