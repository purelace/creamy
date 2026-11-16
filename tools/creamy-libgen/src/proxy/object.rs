use super::EnrichedFieldSymbol;

#[derive(Clone)]
pub struct EnrichedObjectSymbol<'s, I>
where
    I: Iterator<Item = EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    pub fields: I,
}
