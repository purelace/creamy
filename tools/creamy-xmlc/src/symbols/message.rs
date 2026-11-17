#![cfg_attr(coverage_nightly, coverage(off))]

use super::FieldSymbol;
use crate::{
    table::TypeId,
    utils::{Direction, strpool::StringId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapArraySymbol {
    pub(crate) kind: TypeId,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiStatus {
    Stable,
    Extension,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageSymbol {
    pub(crate) ident: StringId,
    pub(crate) fields: Box<[FieldSymbol]>,
    pub(crate) direction: Direction,
    pub(crate) kind: u8,
    pub(crate) api: ApiStatus,
    pub(crate) message_type: MessageSymbolType,
}

impl MessageSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        self.ident
    }

    #[must_use]
    pub const fn fields(&self) -> &[FieldSymbol] {
        &self.fields
    }

    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn kind(&self) -> u8 {
        self.kind
    }

    #[must_use]
    pub const fn message_type(&self) -> &MessageSymbolType {
        &self.message_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutEntry {
    ArrayLength { name: StringId },
    ArrayWrite { name: StringId },
    Slice { to: u64, kind: TypeId },
    LastSlice { to: u64 },
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageSymbolType {
    Single,
    Stream {
        serialization_layout: SerializationLayout,
        lightweight_content: Box<[FieldSymbol]>,
    },
}
