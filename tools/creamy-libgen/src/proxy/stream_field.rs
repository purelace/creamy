use std::borrow::Cow;

use creamy_plugin_package::pool::{StringPool, StringPoolResolver};
use creamy_xmlc::model::{
    FinishedTypeTable,
    layout::LayoutCalculator,
    symbols::{StreamFieldSymbol, StreamFieldType, get_builtin_type_name, is_builtin_type},
};

use crate::{Path, SymbolIterator, utils::AbsolutePath};

#[derive(Clone)]
pub struct FieldMetadata {
    pub offset: usize,
    pub size: usize,
}

pub enum EnrichedStreamFieldType<'s> {
    Type { name: Path },
    StaticArray { kind: Cow<'s, str>, len: u8 },
    DynamicArray { kind: Cow<'s, str> },
}

fn get_padding_kind<'a>(padding: u8) -> EnrichedStreamFieldType<'a> {
    if padding <= 1 {
        EnrichedStreamFieldType::Type {
            name: Path::from_global("u8"),
        }
    } else {
        EnrichedStreamFieldType::StaticArray {
            kind: Cow::Borrowed("u8"),
            len: padding,
        }
    }
}

pub struct EnrichedStreamFieldSymbol<'s> {
    pub name: Cow<'s, str>,
    pub kind: EnrichedStreamFieldType<'s>,
    pub metadata: FieldMetadata,
    pub is_padding: bool,
}

#[derive(Clone)]
pub struct StreamFieldList<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> {
    module_path: AbsolutePath,
    pool: &'s StringPool,
    tt: &'s FinishedTypeTable,
    layout: LayoutCalculator<I::Item, I>,
    padding_index: u8,
    prev: Option<(StreamFieldSymbol, FieldMetadata)>,
    remainder: bool,
}

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> StreamFieldList<'s, I> {
    fn get_field_kind(&self, kind: StreamFieldType) -> EnrichedStreamFieldType<'s> {
        match kind {
            StreamFieldType::Type(id) => {
                let path = if is_builtin_type(id) {
                    let name = get_builtin_type_name(id).resolve(self.pool);
                    Path::from_global(name)
                } else {
                    let component = self.tt.get_type(id).ident().resolve(self.pool);
                    Path::from_absolute(self.module_path.push(component))
                };
                EnrichedStreamFieldType::Type { name: path }
            }
            StreamFieldType::StaticArray(symbol) => {
                let kind = self.tt.get_type(symbol.kind()).ident().resolve(self.pool);
                EnrichedStreamFieldType::StaticArray {
                    kind: Cow::Borrowed(kind),
                    len: symbol.len().value(),
                }
            }
            StreamFieldType::DynamicArray(symbol) => {
                let kind = self.tt.get_type(symbol.kind()).ident().resolve(self.pool);
                EnrichedStreamFieldType::DynamicArray {
                    kind: Cow::Borrowed(kind),
                }
            }
        }
    }
}

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> StreamFieldList<'s, I> {
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

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> Iterator for StreamFieldList<'s, I> {
    type Item = EnrichedStreamFieldSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((prev, metadata)) = self.prev.take() {
            return Some(EnrichedStreamFieldSymbol {
                name: Cow::Borrowed(prev.ident().resolve(self.pool)),
                kind: self.get_field_kind(prev.kind()),
                is_padding: false,
                metadata,
            });
        }

        match self.layout.next() {
            Some((field, step)) => {
                if step.padding == 0 {
                    Some(EnrichedStreamFieldSymbol {
                        name: Cow::Borrowed(field.ident().resolve(self.pool)),
                        kind: self.get_field_kind(field.kind()),
                        is_padding: false,
                        metadata: FieldMetadata {
                            offset: step.offset as usize,
                            size: step.size as usize,
                        },
                    })
                } else {
                    self.prev = Some((
                        field,
                        FieldMetadata {
                            offset: step.offset as usize,
                            size: step.size as usize,
                        },
                    ));

                    let field = EnrichedStreamFieldSymbol {
                        name: Cow::Owned(format!("__padding{}", self.padding_index)),
                        kind: get_padding_kind(step.padding),
                        is_padding: true,
                        metadata: FieldMetadata {
                            offset: step.offset as usize - step.padding as usize,
                            size: step.padding as usize,
                        },
                    };
                    self.padding_index += 1;
                    Some(field)
                }
            }
            //None if !self.remainder => {
            //    //TODO: это костыль ебучий
            //    if self.layout.total_size() > 32 {
            //        self.remainder = true;
            //        return None;
            //    }
            //    self.remainder = true;
            //    let diff = 32 - self.layout.total_size();
            //    if diff != 0 {
            //        let name = Cow::Owned(format!("__padding{}", self.padding_index));
            //        Some(EnrichedStreamFieldSymbol {
            //            name,
            //            //TODO: use get_padding_kind function
            //            kind: self.get_field_kind(StreamFieldType::StaticArray(ArraySymbol::new(
            //                T_U8_ID,
            //                Size::try_new(diff).expect("Unreachable!"),
            //            ))),
            //            is_padding: true,
            //            metadata: FieldMetadata {
            //                offset: 32 - diff as usize,
            //                size: diff as usize,
            //            },
            //        })
            //    } else {
            //        None
            //    }
            //}
            None => None,
        }
    }
}

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone>
    SymbolIterator<EnrichedStreamFieldSymbol<'s>> for StreamFieldList<'s, I>
{
}
