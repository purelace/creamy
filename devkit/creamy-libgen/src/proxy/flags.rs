use creamy_xmlc::{StringPoolResolver, model::symbols::OptionSymbol, utils::strpool::StringPool};

use crate::SymbolIterator;

pub enum FlagUnderlyingType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

pub struct EnrichedFlagsSymbol<'s, I>
where
    I: Iterator<Item = &'s str>,
{
    pub name: &'s str,
    pub underlying_type: FlagUnderlyingType,
    pub options: I,
}

#[derive(Clone)]
pub struct OptionList<'s> {
    pool: &'s StringPool,
    slice: &'s [OptionSymbol],
    index: usize,
}

impl<'s> OptionList<'s> {
    #[must_use]
    pub const fn new(pool: &'s StringPool, slice: &'s [OptionSymbol]) -> Self {
        Self {
            pool,
            slice,
            index: 0,
        }
    }

    #[must_use]
    pub const fn item_count(&self) -> usize {
        self.slice.len()
    }
}

impl<'s> Iterator for OptionList<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.slice.len() {
            return None;
        }

        let item = self.slice[self.index];
        self.index += 1;

        Some(item.ident().resolve(self.pool))
    }
}

impl<'s> SymbolIterator<&'s str> for OptionList<'s> {}
