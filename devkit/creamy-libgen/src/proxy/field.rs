use std::borrow::Cow;

use creamy_xmlc::{
    FinishedTypeTable, StringPoolResolver,
    model::{
        definition::LayoutCalculator,
        symbols::{ArraySymbol, FieldSymbol, FieldType, T_U8_ID},
    },
    utils::{Size, strpool::StringPool},
};

#[derive(Clone)]
pub enum EnrichedFieldType<'s> {
    Type(Cow<'s, str>),
    Array { kind: Cow<'s, str>, len: u8 },
}

fn get_field_kind<'a>(
    kind: FieldType,
    tt: &FinishedTypeTable,
    pool: &'a StringPool,
) -> EnrichedFieldType<'a> {
    match kind {
        FieldType::Type(id) => {
            EnrichedFieldType::Type(Cow::Borrowed(tt.get_type(id).ident().resolve(pool)))
        }
        FieldType::Array(symbol) => {
            let kind = tt.get_type(symbol.kind()).ident().resolve(pool);
            EnrichedFieldType::Array {
                kind: Cow::Borrowed(kind),
                len: symbol.len().value(),
            }
        }
    }
}

#[derive(Clone)]
pub struct EnrichedFieldSymbol<'s> {
    pub name: Cow<'s, str>,
    pub kind: EnrichedFieldType<'s>,
}

#[derive(Clone)]
pub struct FieldList<'s> {
    pool: &'s StringPool,
    tt: &'s FinishedTypeTable,
    layout: LayoutCalculator<'s>,
    padding_index: u8,
    prev: Option<FieldSymbol>,
    remainder: bool,
}

impl<'s> FieldList<'s> {
    pub fn new(
        pool: &'s StringPool,
        tt: &'s FinishedTypeTable,
        fields: &'s [FieldSymbol],
        reserved_bytes: u8,
    ) -> Self {
        Self {
            pool,
            tt,
            layout: LayoutCalculator::new(reserved_bytes, fields),
            padding_index: 0,
            prev: None,
            remainder: false,
        }
    }
}

impl<'s> Iterator for FieldList<'s> {
    type Item = EnrichedFieldSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(prev) = self.prev.take() {
            return Some(EnrichedFieldSymbol {
                name: Cow::Borrowed(prev.name().resolve(self.pool)),
                kind: get_field_kind(prev.kind(), self.tt, self.pool),
            });
        }

        match self.layout.next() {
            Some((field, step)) => {
                if step.padding != 0 {
                    self.prev = Some(field);
                    let field = EnrichedFieldSymbol {
                        name: Cow::Owned(format!("__padding{}", self.padding_index)),
                        kind: get_field_kind(FieldType::Type(T_U8_ID), self.tt, self.pool),
                    };
                    self.padding_index += 1;
                    Some(field)
                } else {
                    Some(EnrichedFieldSymbol {
                        name: Cow::Borrowed(field.name().resolve(self.pool)),
                        kind: get_field_kind(field.kind(), self.tt, self.pool),
                    })
                }
            }
            None if !self.remainder => {
                self.remainder = true;
                let diff = 32 - self.layout.total_size();
                if diff != 0 {
                    let name = Cow::Owned(format!("_padding{}", self.padding_index));
                    Some(EnrichedFieldSymbol {
                        name,
                        kind: get_field_kind(
                            FieldType::Array(ArraySymbol::new(
                                T_U8_ID,
                                Size::new(diff).expect("Unreachable!"),
                            )),
                            self.tt,
                            self.pool,
                        ),
                    })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
