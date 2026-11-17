#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::BinWrite;

use super::FieldSymbol;
use crate::{
    error::Fallback,
    table::TypeId,
    utils::{Direction, strpool::StringId},
};

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapArraySymbol {
    pub kind: TypeId,
}

impl HeapArraySymbol {
    const ARRAY_LENGTH_FIELD: u64 = 4;

    #[must_use]
    pub const fn align(&self) -> u64 {
        Self::ARRAY_LENGTH_FIELD
    }

    #[must_use]
    pub const fn size(&self) -> u64 {
        Self::ARRAY_LENGTH_FIELD
    }
}

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ApiStatus {
    Stable = 0,
    Extension = 1,
    Experimental = 2,
}

impl Fallback for ApiStatus {
    fn fallback() -> Self {
        ApiStatus::Stable
    }
}

#[derive(BinWrite, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageSymbol {
    pub ident: StringId,
    pub fields: Box<[FieldSymbol]>,
    pub direction: Direction,
    pub kind: u8,
    pub api: ApiStatus,
    pub message_type: MessageSymbolType,
}

#[derive(BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LayoutEntry {
    ArrayLength { name: StringId } = 0,
    ArrayWrite { name: StringId } = 1,
    Slice { to: u64, kind: TypeId } = 2,
    LastSlice { to: u64 } = 3,
}

#[derive(BinWrite, Default, Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(BinWrite, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageSymbolType {
    Single = 0,
    Stream {
        serialization_layout: SerializationLayout,
        lightweight_content: Box<[FieldSymbol]>,
    } = 1,
}
