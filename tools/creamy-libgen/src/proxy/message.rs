use std::marker::PhantomData;

use super::EnrichedFieldSymbol;
use crate::SymbolIterator;

#[derive(Clone)]
pub enum EnrichedLayoutEntry<'s> {
    ArrayLength { name: &'s str },
    ArrayWrite { name: &'s str },
    Slice { to: u64, kind: &'s str },
    LastSlice { to: u64 },
}

#[derive(Default, Clone)]
pub struct EnrichedSerializationLayout<'s> {
    pub items: Vec<EnrichedLayoutEntry<'s>>,
}

pub enum MessageType<'s, C>
where
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    Single,
    Stream {
        _phantom: PhantomData<&'s str>,
        serialization_layout: EnrichedSerializationLayout<'s>,
        content_fields: C,
    },
}

pub struct EnrichedMessageSymbol<'s, I, C>
where
    I: SymbolIterator<EnrichedFieldSymbol<'s>>,
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    pub name: &'s str,
    /* Constant Values */
    pub protocol: u8,
    pub kind: u8,
    pub dispatch_value: u32,

    pub fields: I,
    pub message_type: MessageType<'s, C>,
}
