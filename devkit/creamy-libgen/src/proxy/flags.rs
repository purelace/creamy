use creamy_xmlc::{StringPoolResolver, model::symbols::OptionSymbol, utils::strpool::StringPool};

pub struct EnrichedFlagsSymbol<'s, I>
where
    I: Iterator<Item = &'s str>,
{
    pub name: &'s str,
    pub options: I,
    //pub options: OptionList<'s>,
}

#[derive(Clone)]
pub struct OptionList<'s> {
    pool: &'s StringPool,
    slice: &'s [OptionSymbol],
    index: usize,
}

impl<'s> OptionList<'s> {
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

        Some(item.name().resolve(self.pool))
    }
}
