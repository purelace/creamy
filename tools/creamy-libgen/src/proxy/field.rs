use std::borrow::Cow;

use creamy_xmlc::{
    symbols::{FieldSymbol, FieldType, SpecialType, get_builtin_type_name, is_builtin_type},
    table::FinishedSymbolTable,
    utils::strpool::{StringPool, StringPoolResolver},
};

use crate::{Path, SymbolIterator, utils::AbsolutePath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrichedFieldType<'s> {
    Type { name: Path },
    StaticArray { kind: Cow<'s, str>, len: u64 },
    Special(SpecialType),
    HeapArray { kind: Cow<'s, str> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichedFieldSymbol<'s> {
    pub name: Cow<'s, str>,
    pub kind: EnrichedFieldType<'s>,
    pub is_padding: bool,
}

#[derive(Clone)]
pub struct FieldList<'s, I: Iterator<Item = FieldSymbol> + Clone> {
    module_path: AbsolutePath,
    tt: &'s FinishedSymbolTable,
    pool: &'s StringPool,
    fields: I,
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
            FieldType::StaticArray(s) => {
                let kind = self.tt.get_type(s.kind).ident().resolve(self.pool);
                EnrichedFieldType::StaticArray {
                    kind: Cow::Borrowed(kind),
                    len: s.len.value(),
                }
            }
            FieldType::HeapArray(s) => {
                let kind = self.tt.get_type(s.kind).ident().resolve(self.pool);
                EnrichedFieldType::HeapArray {
                    kind: Cow::Borrowed(kind),
                }
            }
            FieldType::Special(s) => EnrichedFieldType::Special(s),
        }
    }
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> FieldList<'s, I> {
    #[must_use]
    pub const fn new(
        module_path: AbsolutePath,
        pool: &'s StringPool,
        tt: &'s FinishedSymbolTable,
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

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> Iterator for FieldList<'s, I> {
    type Item = EnrichedFieldSymbol<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(field) = self.fields.next() {
            Some(EnrichedFieldSymbol {
                name: Cow::Borrowed(field.ident.resolve(self.pool)),
                kind: self.get_field_kind(field.kind),
                is_padding: field.is_padding,
            })
        } else {
            None
        }
    }
}

impl<'s, I: Iterator<Item = FieldSymbol> + Clone> SymbolIterator<EnrichedFieldSymbol<'s>>
    for FieldList<'s, I>
{
}
