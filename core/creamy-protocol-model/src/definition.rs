use as_guard::AsGuard;
use creamy_binary::{BinRead, BinWrite};
pub use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_utils::strpool::{StringId, StringPool};
use semver::Version;

use super::{
    storage::SymbolStorage,
    symbols::{GlobalTypesSymbol, MessageSymbolType, StreamPayloadFieldSymbol},
};
use crate::{
    constraints::{HEADER_BYTES, MAX_PAYLOAD},
    error::SemanticError,
    layout::compute_layout,
    symbols::{BitsetValueSymbol, FieldSymbol, GroupSymbol, OptionSymbol, Type, VariantSymbol},
    table::{FinishedTypeTable, TypeMeta},
    utils::{BitsetValuesRange, FieldsRange, MessagesRange, OptionsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct ProtocolModel {
    name: StringId,
    version: Version,
    global: GlobalTypesSymbol,
    storage: SymbolStorage,
    table: FinishedTypeTable,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ProtocolModel {
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

impl ProtocolModel {
    #[must_use]
    pub fn get_struct_paddings(fields: &[FieldSymbol]) -> u8 {
        let mut paddings = 0;
        compute_layout(HEADER_BYTES, fields, |_, l| {
            paddings += l.padding;
        });
        paddings
    }

    fn get_meta(fields: &[FieldSymbol], reserved: u8) -> Result<TypeMeta, SemanticError> {
        let mut max_align = 1usize;
        let mut total_size = compute_layout(reserved, fields, |_, l| {
            if l.align as usize > max_align {
                max_align = l.align as usize;
            }
        }) as usize;

        let tail_padding = (max_align - (total_size % max_align)) % max_align;
        total_size += tail_padding;
        total_size -= reserved as usize;

        if total_size > MAX_PAYLOAD {
            return Err(SemanticError::InvalidSize { actual: total_size });
        }

        assert!(u8::try_from(max_align).is_ok());

        TypeMeta::try_new(total_size.safe_as::<u8>(), max_align.safe_as::<u8>())
    }

    /// Calculates [`TypeMeta`] for the given slice of fields, reserving [`HEADER_BYTES`] for the message header.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * calculated size > [`MAX_PAYLOAD`]
    pub fn get_message_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(fields, HEADER_BYTES)
    }

    /// Calculates [`TypeMeta`] for the given slice of fields with no bytes reservation.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * calculated size > [`MAX_PAYLOAD`]
    pub fn get_struct_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(fields, 0)
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
