use core::fmt::Display;

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::PrimitiveRepr;
use crate::{
    constraints::{MAX_ENUMS, MAX_VARIANTS},
    error::{Fallback, SemanticError},
    storage::SymbolKey,
    table::TypeMeta,
    utils::{EnumsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Number {
    Signed(i64) = 0,
    Unsigned(u64) = 1,
}

impl Display for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Number::Signed(s) => write!(f, "{s}"),
            Number::Unsigned(u) => write!(f, "{u}"),
        }
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VariantValue {
    Singed(i64) = 0,
    Unsigned(u64) = 1,
}

impl Display for VariantValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VariantValue::Singed(s) => write!(f, "{s}"),
            VariantValue::Unsigned(u) => write!(f, "{u}"),
        }
    }
}

impl Fallback for VariantValue {
    fn fallback() -> Self {
        Self::Unsigned(1)
    }
}

#[derive(Token, Symbol, BinWrite, BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_VARIANTS, VariantsRange)]
#[key(SymbolKey::Variant)]
pub struct VariantSymbol {
    #[token(ident)]
    ident: StringId,
    value: Number,
}

#[derive(Token, BinWrite, BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_ENUMS, EnumsRange)]
pub struct EnumSymbol {
    #[token(ident)]
    ident: StringId,
    repr: PrimitiveRepr,
    variants: VariantsRange,
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
