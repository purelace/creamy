use creamy_xmlc::model::symbols::{Number, PrimitiveRepr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichedVariantSymbol<'s> {
    pub name: &'s str,
    pub value: Number,
}

pub struct EnrichedEnumSymbol<'s, I>
where
    I: Iterator<Item = EnrichedVariantSymbol<'s>>,
{
    pub name: &'s str,
    pub repr: PrimitiveRepr,
    pub variants: I,
}
