use std::borrow::Cow;

use creamy_plugin_package::pool::{StringPool, StringPoolResolver};
use creamy_xmlc::model::{
    FinishedTypeTable, TypeId,
    symbols::{StreamFieldSymbol, StreamFieldType, get_builtin_type_name, is_builtin_type},
};

use crate::{Path, SymbolIterator, utils::AbsolutePath};

#[derive(Debug, PartialEq, Eq)]
pub enum EnrichedStreamFieldType<'s> {
    Type { name: Path },
    StaticArray { kind: Cow<'s, str>, len: u8 },
    DynamicArray { kind: Cow<'s, str> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct EnrichedStreamFieldSymbol<'s> {
    pub name: Cow<'s, str>,
    pub kind: EnrichedStreamFieldType<'s>,
    pub type_id: TypeId,
    pub is_padding: bool,
}

#[derive(Clone)]
pub struct StreamFieldList<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> {
    module_path: AbsolutePath,
    tt: &'s FinishedTypeTable,
    pool: &'s StringPool,
    fields: I,
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
    ) -> Self {
        Self {
            module_path,
            tt,
            pool,
            fields,
        }
    }
}

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone> Iterator for StreamFieldList<'s, I> {
    type Item = EnrichedStreamFieldSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(field) = self.fields.next() {
            Some(EnrichedStreamFieldSymbol {
                name: Cow::Borrowed(field.ident().resolve(self.pool)),
                kind: self.get_field_kind(field.kind()),
                type_id: field.type_id(),
                is_padding: field.is_padding(),
            })
        } else {
            None
        }
    }
}

impl<'s, I: Iterator<Item = StreamFieldSymbol> + Clone>
    SymbolIterator<EnrichedStreamFieldSymbol<'s>> for StreamFieldList<'s, I>
{
}
