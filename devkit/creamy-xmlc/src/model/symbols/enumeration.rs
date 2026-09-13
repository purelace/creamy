use core::fmt::Display;

use binrw::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use crate::{
    constraints::{MAX_ENUMS, MAX_VARIANTS},
    error::{Fallback, SemanticError},
    model::{storage::SymbolKey, symbols::PrimitiveRepr},
    table::TypeMeta,
    utils::{EnumsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantValue {
    #[brw(magic = 0u8)]
    Singed(i64),
    #[brw(magic = 1u8)]
    Unsigned(u64),
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
    value: VariantValue,
}
crate::define_readonly_struct!(@impl_methods VariantSymbol {
    ident: StringId, value: VariantValue,
});

#[derive(Token, BinWrite, BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_ENUMS, EnumsRange)]
pub struct EnumSymbol {
    #[token(ident)]
    ident: StringId,
    repr: PrimitiveRepr,
    variants: VariantsRange,
}
crate::define_readonly_struct!(@impl_methods EnumSymbol {
    ident: StringId, repr: PrimitiveRepr, variants: VariantsRange,
});

impl EnumSymbol {
    pub const fn meta(&self) -> Result<TypeMeta, SemanticError> {
        let ty = self.repr.as_numberic_symbol();
        let size = ty.size();
        let align = ty.align();
        TypeMeta::new(size, align)
    }
}
