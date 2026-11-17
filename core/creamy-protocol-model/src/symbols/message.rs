#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use crate::{
    Direction, TypeId,
    constraints::MAX_MESSAGES,
    storage::SymbolKey,
    utils::{FieldsRange, MessagesRange},
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapArraySymbol {
    kind: TypeId,
}

impl HeapArraySymbol {
    const ARRAY_LENGTH_FIELD: u64 = 4;

    #[must_use]
    pub const fn new(kind: TypeId) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> TypeId {
        self.kind
    }

    #[must_use]
    pub const fn align(&self) -> u64 {
        Self::ARRAY_LENGTH_FIELD
    }

    #[must_use]
    pub const fn size(&self) -> u64 {
        Self::ARRAY_LENGTH_FIELD
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ApiStatus {
    Stable = 0,
    Extension = 1,
    Experimental = 2,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
#[key(SymbolKey::Message)]
pub struct MessageSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
    direction: Direction,
    kind: u8,
    api: ApiStatus,
    message_type: MessageSymbolType,
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LayoutEntry {
    ArrayLength { name: StringId } = 0,
    ArrayWrite { name: StringId } = 1,
    Slice { to: u64, kind: TypeId } = 2,
    LastSlice { to: u64 } = 3,
}

#[derive(Default, BinRead, BinWrite, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializationLayout {
    items: Vec<LayoutEntry>,
}

impl SerializationLayout {
    #[must_use]
    pub const fn items(&self) -> &[LayoutEntry] {
        self.items.as_slice()
    }

    pub fn push(&mut self, layout: LayoutEntry) {
        self.items.push(layout);
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageSymbolType {
    Single = 0,
    Stream {
        serialization_layout: SerializationLayout,
        lightweight_content: FieldsRange,
    } = 1,
}
