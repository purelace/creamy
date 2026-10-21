#![cfg_attr(coverage_nightly, coverage(off))]

use creamy_binary::{BinRead, BinWrite};
use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::{ArraySymbol, Field};
use crate::{
    Align, Direction, TypeId, TypeMeta,
    constraints::{MAX_FIELDS, MAX_MESSAGES},
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
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StreamFieldType {
    Type(TypeId) = 0,
    StaticArray(ArraySymbol) = 1,
    DynamicArray(HeapArraySymbol) = 2,
}

#[derive(BinRead, BinWrite, Token, Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_FIELDS, FieldsRange)]
#[key(SymbolKey::StreamField)]
pub struct StreamFieldSymbol {
    #[token(ident)]
    ident: StringId,
    kind: StreamFieldType,
}

impl Field for StreamFieldSymbol {
    fn type_id(&self) -> TypeId {
        let type_id = self.type_id();
        let group = type_id.group();
        let index = type_id.index();
        let size = type_id.size();
        TypeId::new(group, StringId::new(index), TypeMeta::new(size, Align::B4))
    }

    fn size(&self) -> u8 {
        match self.kind {
            StreamFieldType::Type(sym) => sym.size().value(),
            StreamFieldType::StaticArray(array) => array.total_size(),
            StreamFieldType::DynamicArray(_) => {
                const ARRAY_LENGTH_FIELD: u8 = 4;
                const ARRAY_POINTER_FIELD: u8 = 4;

                ARRAY_LENGTH_FIELD + ARRAY_POINTER_FIELD
            }
        }
    }
}

impl StreamFieldSymbol {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self.kind {
            StreamFieldType::Type(sym) => sym,
            StreamFieldType::StaticArray(sym) => sym.kind(),
            StreamFieldType::DynamicArray(sym) => sym.kind(),
        }
    }
}

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

    /// Content of the message stream.
    content: FieldsRange,
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
