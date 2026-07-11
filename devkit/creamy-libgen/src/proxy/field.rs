use std::borrow::Cow;

use creamy_xmlc::{
    FinishedTypeTable, StringPoolResolver,
    model::{
        definition::LayoutCalculator,
        symbols::{
            ArraySymbol, FieldSymbol, FieldType, T_U8_ID, get_builtin_type_name, is_builtin_type,
        },
    },
    utils::{Size, strpool::StringPool},
};

use crate::{Path, SymbolIterator, utils::AbsolutePath};

#[derive(Clone)]
pub enum EnrichedFieldType<'s> {
    Type { name: Path },
    Array { kind: Cow<'s, str>, len: u8 },
}

fn get_padding_kind<'a>(padding: u8) -> EnrichedFieldType<'a> {
    if padding <= 1 {
        EnrichedFieldType::Type {
            name: Path::from_global("u8"),
        }
    } else {
        EnrichedFieldType::Array {
            kind: Cow::Borrowed("u8"),
            len: padding,
        }
    }
}

//fn get_field_kind<'a>(
//    kind: FieldType,
//    tt: &FinishedTypeTable,
//    pool: &'a StringPool,
//) -> EnrichedFieldType<'a> {
//    match kind {
//        FieldType::Type(id) => EnrichedFieldType::Type {
//            name: Cow::Borrowed(tt.get_type(id).ident().resolve(pool)),
//        },
//        FieldType::Array(symbol) => {
//            let kind = tt.get_type(symbol.kind()).ident().resolve(pool);
//            EnrichedFieldType::Array {
//                kind: Cow::Borrowed(kind),
//                len: symbol.len().value(),
//            }
//        }
//    }
//}

#[derive(Clone)]
pub struct EnrichedFieldSymbol<'s> {
    pub name: Cow<'s, str>,
    pub kind: EnrichedFieldType<'s>,
    pub is_padding: bool,
}

#[derive(Clone)]
pub struct FieldList<'s, I: Iterator<Item = FieldSymbol> + Clone> {
    module_path: AbsolutePath,
    pool: &'s StringPool,
    tt: &'s FinishedTypeTable,
    layout: LayoutCalculator<I>,
    padding_index: u8,
    prev: Option<FieldSymbol>,
    remainder: bool,
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> FieldList<'s, I> {
    fn get_field_kind(&self, kind: FieldType) -> EnrichedFieldType<'s> {
        match kind {
            FieldType::Type(id) => {
                let path = if is_builtin_type(id) {
                    let name = get_builtin_type_name(id).resolve(self.pool);
                    Path::from_global(name)
                } else {
                    let component = self.tt.get_type(id).ident().resolve(self.pool);
                    Path::from_absolute(self.module_path.push(component))
                };
                EnrichedFieldType::Type { name: path }
            }
            FieldType::Array(symbol) => {
                let kind = self.tt.get_type(symbol.kind()).ident().resolve(self.pool);
                EnrichedFieldType::Array {
                    kind: Cow::Borrowed(kind),
                    len: symbol.len().value(),
                }
            }
        }
    }
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> FieldList<'s, I> {
    #[must_use]
    pub const fn new(
        module_path: AbsolutePath,
        pool: &'s StringPool,
        tt: &'s FinishedTypeTable,
        fields: I,
        reserved_bytes: u8,
    ) -> Self {
        Self {
            module_path,
            pool,
            tt,
            layout: LayoutCalculator::new(reserved_bytes, fields),
            padding_index: 0,
            prev: None,
            remainder: false,
        }
    }
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> Iterator for FieldList<'s, I> {
    type Item = EnrichedFieldSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(prev) = self.prev.take() {
            return Some(EnrichedFieldSymbol {
                name: Cow::Borrowed(prev.name().resolve(self.pool)),
                kind: self.get_field_kind(prev.kind()),
                is_padding: false,
            });
        }

        match self.layout.next() {
            Some((field, step)) => {
                if step.padding != 0 {
                    self.prev = Some(field);
                    let field = EnrichedFieldSymbol {
                        name: Cow::Owned(format!("__padding{}", self.padding_index)),
                        kind: get_padding_kind(step.padding),
                        is_padding: true,
                    };
                    self.padding_index += 1;
                    Some(field)
                } else {
                    Some(EnrichedFieldSymbol {
                        name: Cow::Borrowed(field.name().resolve(self.pool)),
                        kind: self.get_field_kind(field.kind()),
                        is_padding: false,
                    })
                }
            }
            None if !self.remainder => {
                self.remainder = true;
                let diff = 32 - self.layout.total_size();
                if diff != 0 {
                    let name = Cow::Owned(format!("__padding{}", self.padding_index));
                    Some(EnrichedFieldSymbol {
                        name,
                        //TODO: use get_padding_kind function
                        kind: self.get_field_kind(FieldType::Array(ArraySymbol::new(
                            T_U8_ID,
                            Size::new(diff).expect("Unreachable!"),
                        ))),
                        is_padding: true,
                    })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> SymbolIterator<EnrichedFieldSymbol<'s>>
    for FieldList<'s, I>
{
}
