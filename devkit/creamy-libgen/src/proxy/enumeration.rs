use creamy_xmlc::{VariantValue, model::symbols::EnumRepr};

pub struct ResolvedVariant<'s> {
    pub name: &'s str,
    pub value: VariantValue,
}

pub struct EnrichedEnumSymbol<'s, I>
where
    I: Iterator<Item = ResolvedVariant<'s>>,
{
    pub name: &'s str,
    pub repr: EnumRepr,
    pub variants: I,
}
