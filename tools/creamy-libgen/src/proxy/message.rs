use std::marker::PhantomData;

use super::EnrichedFieldSymbol;
use crate::SymbolIterator;

pub enum MessageType<'s, C>
where
    C: SymbolIterator<EnrichedFieldSymbol<'s>>,
{
    Single,
    Stream {
        _phantom: PhantomData<&'s str>,
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
