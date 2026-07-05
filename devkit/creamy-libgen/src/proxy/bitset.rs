pub struct EnrichedBitsetValue<'s> {
    pub name: &'s str,
    pub bits: u8,
    pub repr: &'s str,
}

pub struct EnrichedBitsetSymbol<'s> {
    pub name: &'s str,
}
