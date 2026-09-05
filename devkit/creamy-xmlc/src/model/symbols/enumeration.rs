use creamy_utils::strpool::StringId;

use crate::{
    VariantValue,
    constraints::{MAX_ENUMS, MAX_VARIANTS},
    define_readonly_struct,
    error::SemanticError,
    impl_with_ident,
    model::{
        storage::{Symbol, SymbolKey},
        symbols::PrimitiveRepr,
    },
    table::TypeMeta,
    utils::{EnumsRange, VariantsRange},
};

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
