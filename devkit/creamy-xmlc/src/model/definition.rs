use std::{fmt::Display, str::FromStr};

use as_guard::AsGuard;
use binrw::{BinRead, BinWrite};
use creamy_utils::{
    BString,
    strpool::{StringId, StringPool},
};
use semver::Version;

use crate::{
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    error::{Fallback, SemanticError},
    model::symbols::{
        BitsetValueSymbol, FieldSymbol, FieldType, GroupSymbol, MessageSymbolType, OptionSymbol,
        StreamPayloadFieldSymbol, Type, VariantSymbol,
    },
    table::{FinishedTypeTable, TypeMeta},
    utils::{BValuesRange, BoundedVec, FieldsRange, MessagesRange, OptionsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    #[default]
    #[brw(magic(1u8))]
    /// Все пишут, все читают
    Public,

    #[brw(magic(2u8))]
    /// Читают все, пишет один
    Protected,

    #[brw(magic(3u8))]
    /// Читает один, пишут все
    Private,

    #[brw(magic(4u8))]
    /// Читает один, пишет один
    Exclusive,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Access::Public => "Public",
            Access::Protected => "Protected",
            Access::Private => "Private",
            Access::Exclusive => "Exclusive",
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
    global: GroupSymbol,

    groups: BoundedVec<GroupSymbol>,
    messages: BoundedVec<MessageSymbolType>,

    values: BoundedVec<BitsetValueSymbol>,
    options: BoundedVec<OptionSymbol>,
    variants: BoundedVec<VariantSymbol>,
    fields: BoundedVec<FieldSymbol>,
    payload: BoundedVec<StreamPayloadFieldSymbol>,

    table: FinishedTypeTable,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ProtocolDefinition {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        name: StringId,
        version: Version,
        global: GroupSymbol,

        groups: BoundedVec<GroupSymbol>,
        messages: BoundedVec<MessageSymbolType>,

        values: BoundedVec<BitsetValueSymbol>,
        options: BoundedVec<OptionSymbol>,
        variants: BoundedVec<VariantSymbol>,
        fields: BoundedVec<FieldSymbol>,
        payload: BoundedVec<StreamPayloadFieldSymbol>,

        table: FinishedTypeTable,
    ) -> Self {
        Self {
            name,
            version,
            global,
            groups,
            messages,
            values,
            options,
            variants,
            fields,
            payload,
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
    pub const fn global(&self) -> GroupSymbol {
        self.global
    }

    #[must_use]
    pub fn groups(&self) -> &[GroupSymbol] {
        self.groups.as_slice()
    }

    #[must_use]
    pub fn messages(&self) -> &[MessageSymbolType] {
        self.messages.as_slice()
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
    pub fn messages_slice(&self, messages: MessagesRange) -> &[MessageSymbolType] {
        &self.messages[messages]
    }

    #[must_use]
    pub fn fields_slice(&self, fields: FieldsRange) -> &[FieldSymbol] {
        &self.fields[fields]
    }

    #[must_use]
    pub fn payload_slice(&self, fields: FieldsRange) -> &[StreamPayloadFieldSymbol] {
        &self.payload[fields]
    }

    #[must_use]
    pub fn variants_slice(&self, variants: VariantsRange) -> &[VariantSymbol] {
        &self.variants[variants]
    }

    #[must_use]
    pub fn bvalues_slice(&self, bvalues: BValuesRange) -> &[BitsetValueSymbol] {
        &self.values[bvalues]
    }

    #[must_use]
    pub fn options_slice(&self, options: OptionsRange) -> &[OptionSymbol] {
        &self.options[options]
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
        self.groups
            .iter()
            .map(|g| (g, &self.messages[g.messages()], self.types_for_group(*g)))
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
        //name: StringId,
        //version: Version,
        //global: GroupSymbol,

        //groups: BoundedVec<GroupSymbol>,
        //messages: BoundedVec<MessageSymbolType>,

        //values: BoundedVec<BitsetValueSymbol>,
        //options: BoundedVec<OptionSymbol>,
        //variants: BoundedVec<VariantSymbol>,
        //fields: BoundedVec<FieldSymbol>,
        //payload: BoundedVec<StreamPayloadFieldSymbol>,

        //table: FinishedTypeTable,

        replace!(self.name);
        replace_symbol!(&mut self.global);

        for symbol in self.groups.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.messages.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.values.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.options.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.variants.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.fields.iter_mut() {
            replace_symbol!(symbol);
        }

        for symbol in self.payload.iter_mut() {
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
