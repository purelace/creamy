use creamy_binary::{BinRead, BinWrite};
pub use creamy_binary::{BinaryReader, BinaryWriter};
use creamy_utils::strpool::{StringId, StringPool};
use semver::Version;

use super::storage::SymbolStorage;
use crate::{
    Access,
    constraints::MAX_PAYLOAD,
    error::SemanticError,
    layout::compute_layout,
    symbols::{BitsetValueSymbol, FieldSymbol, MessageSymbol, OptionSymbol, Type, VariantSymbol},
    table::{FinishedSymbolTable, TypeMeta},
    utils::{BitsetValuesRange, FieldsRange, OptionsRange, VariantsRange},
};

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct ProtocolModel {
    name: StringId,
    version: Version,
    access: Access,
    storage: SymbolStorage,
    table: FinishedSymbolTable,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ProtocolModel {
    #[must_use]
    pub const fn new(
        name: StringId,
        version: Version,
        access: Access,
        storage: SymbolStorage,
        table: FinishedSymbolTable,
    ) -> Self {
        Self {
            name,
            version,
            access,
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
    pub const fn access(&self) -> Access {
        self.access
    }

    #[must_use]
    pub fn messages(&self) -> &[MessageSymbol] {
        self.storage.get_symbol_slice::<MessageSymbol>()
    }

    #[must_use]
    pub fn types(&self) -> &[Type] {
        self.table.types()
    }

    #[must_use]
    pub const fn table(&self) -> &FinishedSymbolTable {
        &self.table
    }

    #[must_use]
    pub fn fields_range(&self, range: FieldsRange) -> &[FieldSymbol] {
        self.storage.get_symbol_range::<FieldSymbol>(range)
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
    fn get_meta(
        //TODO: align
        max_size: Option<u8>,
        fields: &[FieldSymbol],
    ) -> Result<TypeMeta, SemanticError> {
        let mut max_align = 1u64;
        let mut total_size = compute_layout(max_size, fields, |_, l| {
            if l.align > max_align {
                max_align = l.align;
            }
        });

        let tail_padding = (max_align - (total_size % max_align)) % max_align;
        total_size += tail_padding;

        if total_size > MAX_PAYLOAD as u64 {
            return Err(SemanticError::InvalidSize { actual: total_size });
        }

        assert!(u8::try_from(max_align).is_ok());

        TypeMeta::try_new(total_size, max_align)
    }

    /// Calculates [`TypeMeta`] for the given slice of fields, reserving [`HEADER_BYTES`] for the message header.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * calculated size > [`MAX_PAYLOAD`]
    pub fn get_message_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(Some(32), fields)
    }

    /// Calculates [`TypeMeta`] for the given slice of fields with no bytes reservation.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * calculated size > [`MAX_PAYLOAD`]
    pub fn get_struct_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, SemanticError> {
        Self::get_meta(None, fields)
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

        for symbol in self.storage.get_symbol_slice_mut::<MessageSymbol>() {
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

        self.table.replace_identifiers(from, to);
    }
}
