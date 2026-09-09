use std::fmt::Display;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;

use crate::{
    constraints::{MAX_ENUMS, MAX_VARIANTS},
    define_readonly_struct,
    error::{Fallback, SemanticError},
    impl_with_ident,
    model::{
        storage::{Symbol, SymbolKey},
        symbols::PrimitiveRepr,
    },
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

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for VariantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

define_readonly_struct! {
    [element(MAX_VARIANTS, VariantsRange)]
    struct VariantSymbol {
        ident: StringId,
        value: VariantValue,
    }
}
impl_with_ident!(VariantSymbol);

impl Symbol for VariantSymbol {
    const KEY: SymbolKey = SymbolKey::Variant;
}

define_readonly_struct! {
    [element(MAX_ENUMS, EnumsRange)]
    struct EnumSymbol {
        ident: StringId,
        repr: PrimitiveRepr,
        variants: VariantsRange,
    }
}
impl_with_ident!(EnumSymbol);

impl EnumSymbol {
    pub const fn meta(&self) -> Result<TypeMeta, SemanticError> {
        let ty = self.repr.as_numberic_symbol();
        let size = ty.size();
        let align = ty.align();
        TypeMeta::new(size, align)
    }
}
