use std::{fmt::Display, str::FromStr};

use as_guard::AsGuard;
use binrw::{BinRead, BinWrite};
use creamy_utils::{
    BString,
    strpool::{StringId, StringPool},
};
use semver::Version;

use super::{storage::SymbolStorage, symbols::GlobalTypesSymbol};
use crate::{
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    error::{Fallback, SemanticError},
    model::symbols::{
        BitsetValueSymbol, FieldSymbol, FieldType, GroupSymbol, MessageSymbolType, OptionSymbol,
        StreamPayloadFieldSymbol, Type, VariantSymbol,
    },
    table::{FinishedTypeTable, TypeMeta},
    utils::{BitsetValuesRange, FieldsRange, MessagesRange, OptionsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    #[brw(magic(0u8))]
    /// Один поставщик - много пользователей
    Public,

    #[default]
    #[brw(magic(1u8))]
    /// Один поставщик - один пользователей
    Private,
}

impl FromStr for Access {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Public" => Ok(Access::Public),
            "Private" => Ok(Access::Private),
            _ => Err(()),
        }
    }
}

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    #[default]
    #[brw(magic = 0u8)]
    Incoming,
    #[brw(magic = 1u8)]
    Outgoing,
    #[brw(magic = 2u8)]
    Duplex,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Incoming" => Ok(Direction::Incoming),
            "Outgoing" => Ok(Direction::Outgoing),
            "Duplex" => Ok(Direction::Duplex),
            _ => Err(()),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Access::Public => "Public",
            Access::Private => "Private",
        };
        write!(f, "{str}")
    }
}

impl Fallback for Access {
    fn fallback() -> Self {
        Self::default()
    }
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct ProtocolDefinition {
    name: StringId,
    #[br(map = |val: BString| Version::from_str(&val).unwrap())]
    #[bw(map = |val: &Version| BString::wrap(val.to_string()))]
    version: Version,
    global: GlobalTypesSymbol,
    storage: SymbolStorage,
    table: FinishedTypeTable,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ProtocolDefinition {
    #[must_use]
    pub const fn new(
        name: StringId,
        version: Version,
        global: GlobalTypesSymbol,
        storage: SymbolStorage,
        table: FinishedTypeTable,
    ) -> Self {
        Self {
            name,
            version,
            global,
            storage,
            table,
        }
    }

    #[must_use]
    pub const fn name(&self) -> StringId {
        self.name
    }

    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub const fn global_types(&self) -> GlobalTypesSymbol {
        self.global
    }

    #[must_use]
    pub fn groups(&self) -> &[GroupSymbol] {
        self.storage.get_symbol_slice::<GroupSymbol>()
    }

    #[must_use]
    pub fn messages(&self) -> &[MessageSymbolType] {
        self.storage.get_symbol_slice::<MessageSymbolType>()
    }

    #[must_use]
    pub const fn table(&self) -> &FinishedTypeTable {
        &self.table
    }

    #[must_use]
    pub fn types_for_group(&self, group: GroupSymbol) -> &[Type] {
        &self.table[group.types()]
    }

    #[must_use]
    pub fn global_types_range(&self) -> &[Type] {
        &self.table[self.global.types()]
    }

    #[must_use]
    pub fn message_range(&self, range: MessagesRange) -> &[MessageSymbolType] {
        self.storage.get_symbol_range(range)
    }

    #[must_use]
    pub fn fields_range(&self, range: FieldsRange) -> &[FieldSymbol] {
        self.storage.get_symbol_range::<FieldSymbol>(range)
    }

    #[must_use]
    pub fn payload_slice(&self, range: FieldsRange) -> &[StreamPayloadFieldSymbol] {
        self.storage
            .get_symbol_range::<StreamPayloadFieldSymbol>(range)
    }

    #[must_use]
    pub fn variants_slice(&self, range: VariantsRange) -> &[VariantSymbol] {
        self.storage.get_symbol_range::<VariantSymbol>(range)
    }

    #[must_use]
    pub fn bvalues_slice(&self, range: BitsetValuesRange) -> &[BitsetValueSymbol] {
        self.storage.get_symbol_range::<BitsetValueSymbol>(range)
    }

    #[must_use]
    pub fn options_slice(&self, range: OptionsRange) -> &[OptionSymbol] {
        self.storage.get_symbol_range::<OptionSymbol>(range)
    }
}

impl ProtocolDefinition {
    #[must_use]
    pub fn get_struct_paddings(fields: &[FieldSymbol]) -> u8 {
        let mut paddings = 0;
        compute_layout::<()>(HEADER_BYTES, fields, |_, l| {
            paddings += l.padding;
            Ok(())
        })
        .unwrap();
        paddings
    }

    fn get_meta(fields: &[FieldSymbol], reserved: u8) -> Result<TypeMeta, SemanticError> {
        let mut max_align = 1usize;
        let mut total_size = compute_layout::<()>(reserved, fields, |_, l| {
            if l.align as usize > max_align {
                max_align = l.align as usize;
            }

            Ok(())
        })
        .unwrap() as usize;

        let tail_padding = (max_align - (total_size % max_align)) % max_align;
        total_size += tail_padding;
        total_size -= reserved as usize;

        if total_size > MAX_PAYLOAD {
            return Err(SemanticError::InvalidSize { actual: total_size });
        }

        assert!(u8::try_from(max_align).is_ok());

        TypeMeta::new(total_size.safe_as::<u8>(), max_align.safe_as::<u8>())
    }

    pub fn get_message_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(fields, HEADER_BYTES)
    }

    pub fn get_struct_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(fields, 0)
    }

    pub fn group_iter<E>(
        &self,
        mut f: impl FnMut(GroupSymbol, &[MessageSymbolType], &[Type]) -> Result<(), E>,
    ) -> Result<(), E> {
        self.groups()
            .iter()
            .map(|g| {
                (
                    g,
                    self.message_range(g.messages()),
                    self.types_for_group(*g),
                )
            })
            .try_for_each(|(g, m, t)| f(*g, m, t))
    }

    pub fn replace_identifiers(&mut self, from: &StringPool, to: &mut StringPool) {
        macro_rules! replace {
            ($field:expr) => {
                let value = from.get_string($field);
                let id = to.get_id_or_add(value);
                $field = id;
            };
        }

        macro_rules! replace_symbol {
            ($symbol:expr) => {
                let value = from.get_string($symbol.ident());
                let id = to.get_id_or_add(value);
                *$symbol = $symbol.with_ident(id);
            };
        }

        replace!(self.name);
        //replace_symbol!(&mut self.global);

        for symbol in self.storage.get_symbol_slice_mut::<GroupSymbol>() {
            replace_symbol!(symbol);
        }

        for symbol in self.storage.get_symbol_slice_mut::<MessageSymbolType>() {
            replace_symbol!(symbol);
        }

        for symbol in self.storage.get_symbol_slice_mut::<BitsetValueSymbol>() {
            replace_symbol!(symbol);
        }

        for symbol in self.storage.get_symbol_slice_mut::<OptionSymbol>() {
            replace_symbol!(symbol);
        }

        for symbol in self.storage.get_symbol_slice_mut::<VariantSymbol>() {
            replace_symbol!(symbol);
        }

        for symbol in self.storage.get_symbol_slice_mut::<FieldSymbol>() {
            replace_symbol!(symbol);
        }

        for symbol in self
            .storage
            .get_symbol_slice_mut::<StreamPayloadFieldSymbol>()
        {
            replace_symbol!(symbol);
        }

        self.table.replace_identifiers(from, to);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LayoutStep {
    pub size: u8,
    pub align: u8,
    pub padding: u8,
    pub offset: u8,
}

#[derive(Clone)]
pub struct LayoutCalculator<I: Iterator<Item = FieldSymbol>> {
    total_size: u8,
    fields: I,
}

impl<I: Iterator<Item = FieldSymbol>> LayoutCalculator<I> {
    #[must_use]
    pub const fn new(reserved: u8, fields: I) -> Self {
        Self {
            total_size: reserved,
            fields,
        }
    }

    #[must_use]
    pub const fn total_size(&self) -> u8 {
        self.total_size
    }
}

impl<I: Iterator<Item = FieldSymbol>> Iterator for LayoutCalculator<I> {
    type Item = (FieldSymbol, LayoutStep);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(field) = self.fields.next() {
            let type_id = field.type_id();
            let align = type_id.align().value();

            let size = match field.kind() {
                FieldType::Type(sym) => sym.size().value(),
                FieldType::Array(array) => {
                    let kind_size = array.kind().size().value();
                    let array_len = array.len().value();
                    kind_size * array_len
                }
            };

            let padding = (align - (self.total_size % align)) % align;

            let step = LayoutStep {
                size,
                align,
                padding,
                offset: self.total_size + padding,
            };

            //TODO: overflow
            self.total_size += padding + size;

            Some((field, step))
        } else {
            None
        }
    }
}

pub fn compute_layout<E>(
    reserved: u8,
    fields: &[FieldSymbol],
    mut f: impl FnMut(&FieldSymbol, LayoutStep) -> Result<(), E>,
) -> Result<u8, E> {
    let mut total_size = reserved;

    for field in fields {
        let type_id = field.type_id();
        let align = type_id.align().value();

        let size = match field.kind() {
            FieldType::Type(sym) => sym.size().value(),
            FieldType::Array(array) => {
                let kind_size = array.kind().size().value();
                let array_len = array.len().value();
                kind_size * array_len
            }
        };

        let padding = (align - (total_size % align)) % align;

        f(
            field,
            LayoutStep {
                size,
                align,
                padding,
                offset: total_size + padding,
            },
        )?;

        //TODO: overflow
        total_size += padding + size;
    }

    Ok(total_size)
}
