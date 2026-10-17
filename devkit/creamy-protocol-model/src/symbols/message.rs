#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::{ArrayFieldSymbol, FieldSymbol};
use crate::{
    Direction,
    constraints::{MAX_FIELDS, MAX_MESSAGES},
    storage::SymbolKey,
    utils::{FieldsRange, MessagesRange, VectorElement},
};

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
pub struct MessageSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
    direction: Direction,
    kind: u8,
}

#[derive(BinRead, BinWrite, Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
pub struct StreamSymbol {
    #[token(ident)]
    /// Name of message.
    ident: StringId,

    /// Direction of the message
    direction: Direction,

    /// Timeout in frames
    timeout: u8,

    /// Type of the message stream.
    /// Allowed types: `[ Head, Payload, Tail ]`
    kind: u8,

    /// Head of the message stream.
    head: Option<FieldsRange>,

    /// Payload of the message stream.
    payload: FieldsRange,

    /// Tail of the message stream
    tail: Option<FieldsRange>,
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
// TODO:
// Тут надо пересчитать максимальное количество типов.
#[derive(BinRead, BinWrite, Symbol, Debug, Clone, Copy, PartialEq, Eq)]
#[key(SymbolKey::StreamPayloadField)]
#[repr(u8)]
pub enum StreamPayloadFieldSymbol {
    Field(FieldSymbol) = 0,
    Array(ArrayFieldSymbol) = 1,
}

impl VectorElement for StreamPayloadFieldSymbol {
    const MAX_SIZE: usize = MAX_FIELDS;
    type RangeType = FieldsRange;
}

impl StreamPayloadFieldSymbol {
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            StreamPayloadFieldSymbol::Field(s) => s.ident(),
            StreamPayloadFieldSymbol::Array(s) => s.ident(),
        }
    }

    #[must_use]
    pub const fn with_ident(&self, id: StringId) -> Self {
        match self {
            StreamPayloadFieldSymbol::Field(s) => StreamPayloadFieldSymbol::Field(s.with_ident(id)),
            StreamPayloadFieldSymbol::Array(s) => StreamPayloadFieldSymbol::Array(s.with_ident(id)),
        }
    }
}
