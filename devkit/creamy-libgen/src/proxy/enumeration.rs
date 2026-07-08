use creamy_xmlc::{VariantValue, model::symbols::PrimitiveRepr};

#[derive(Clone)]
pub struct EnrichedVariantSymbol<'s> {
    pub name: &'s str,
    pub value: VariantValue,
}

pub struct EnrichedEnumSymbol<'s, I>
where
    I: Iterator<Item = EnrichedVariantSymbol<'s>>,
{
    pub name: &'s str,
    pub repr: PrimitiveRepr,
    pub variants: I,
}
