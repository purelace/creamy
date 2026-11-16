#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use crate::{
    Direction, TypeId,
    constraints::MAX_MESSAGES,
    storage::SymbolKey,
    utils::{FieldsRange, MessagesRange, VectorElement},
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapArraySymbol {
    kind: TypeId,
}

impl HeapArraySymbol {
    #[must_use]
    pub const fn new(kind: TypeId) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> TypeId {
        self.kind
    }

    #[must_use]
    pub const fn size(&self) -> u64 {
        const ARRAY_LENGTH_FIELD: u64 = 4;
        ARRAY_LENGTH_FIELD
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ApiStatus {
    Stable = 0,
    Extension = 1,
    Experimental = 2,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
pub struct MessageSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
    direction: Direction,
    kind: u8,
    api: ApiStatus,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
pub struct StreamSymbol {
    #[token(ident)]
    /// Name of message.
    ident: StringId,

    /// Direction of the message
    direction: Direction,

    /// Type of the message stream.
    kind: u8,

    /// Content of the message stream.
    content: FieldsRange,

    /// Api status of the message
    api: ApiStatus,
}

#[derive(BinRead, BinWrite, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[key(SymbolKey::Message)]
#[repr(u8)]
pub enum MessageSymbolType {
    Single(MessageSymbol) = 0,
    Stream(StreamSymbol) = 1,
}

impl VectorElement for MessageSymbolType {
    const MAX_SIZE: usize = MAX_MESSAGES;
    type RangeType = MessagesRange;
}

impl MessageSymbolType {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            MessageSymbolType::Single(m) => m.ident(),
            MessageSymbolType::Stream(s) => s.ident(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> u8 {
        match self {
            MessageSymbolType::Single(m) => m.kind,
            MessageSymbolType::Stream(m) => m.kind,
        }
    }

    #[must_use]
    pub const fn direction(&self) -> Direction {
        match self {
            MessageSymbolType::Single(m) => m.direction,
            MessageSymbolType::Stream(m) => m.direction,
        }
    }

    #[must_use]
    pub const fn with_ident(&self, id: StringId) -> Self {
        match self {
            MessageSymbolType::Single(m) => MessageSymbolType::Single(m.with_ident(id)),
            MessageSymbolType::Stream(m) => MessageSymbolType::Stream(m.with_ident(id)),
        }
    }
}
