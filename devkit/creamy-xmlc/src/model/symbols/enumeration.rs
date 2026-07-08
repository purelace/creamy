use creamy_utils::strpool::StringId;

use crate::{
    constraints::{MAX_ENUMS, MAX_VARIANTS},
    define_readonly_struct,
    error::SemanticError,
    model::symbols::PrimitiveRepr,
    nodes::VariantValue,
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

define_readonly_struct! {
    [element(MAX_ENUMS, EnumsRange)]
    struct EnumSymbol {
        name: StringId,
        repr: PrimitiveRepr,
        variants: VariantsRange,
    }
}

impl EnumSymbol {
    pub const fn meta(&self) -> Result<TypeMeta, SemanticError> {
        let ty = self.repr.as_numberic_symbol();
        let size = ty.size();
        let align = ty.align();
        TypeMeta::new(size, align)
    }
}
