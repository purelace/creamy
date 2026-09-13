use creamy_protocol_model_macros::{Symbol, Token};
use creamy_utils::strpool::StringId;

use super::{ArrayFieldSymbol, FieldSymbol};
use crate::{
    constraints::{MAX_FIELDS, MAX_MESSAGES},
    model::{storage::SymbolKey, utils::Direction},
    utils::{FieldsRange, MessagesRange, VectorElement},
};

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[element(MAX_MESSAGES, MessagesRange)]
pub struct MessageSymbol {
    #[token(ident)]
    ident: StringId,
    fields: FieldsRange,
    direction: Direction,
    kind: u8,
}

crate::define_readonly_struct!(@impl_methods MessageSymbol {
    ident: StringId, fields: FieldsRange, direction: Direction, kind: u8,
});

#[binrw::binrw]
#[derive(Token, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[br(parse_with = read_opt)]
    #[bw(write_with = write_opt)]
    head: Option<FieldsRange>,

    /// Payload of the message stream.
    payload: FieldsRange,

    /// Tail of the message stream
    #[br(parse_with = read_opt)]
    #[bw(write_with = write_opt)]
    tail: Option<FieldsRange>,
}
crate::define_readonly_struct!(@impl_methods StreamSymbol {
    ident: StringId,
    direction: Direction,
    timeout: u8,
    kind: u8,
    head: Option<FieldsRange>,
    payload: FieldsRange,
    tail: Option<FieldsRange>,
});

fn read_opt<T: binrw::BinRead<Args<'static> = ()>, R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    endian: binrw::Endian,
    _: (),
) -> binrw::BinResult<Option<T>> {
    use binrw::BinRead;
    let has_pos: u8 = <u8>::read_options(reader, endian, ())?;
    if has_pos != 0 {
        let data = <T>::read_options(reader, endian, ())?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

#[allow(clippy::ref_option)]
fn write_opt<T: binrw::BinWrite<Args<'static> = ()>, W: std::io::Write + std::io::Seek>(
    opt: &Option<T>,
    writer: &mut W,
    endian: binrw::Endian,
    _: (),
) -> binrw::BinResult<()> {
    use binrw::BinWrite;
    if let Some(data) = opt {
        <u8>::write_options(&1u8, writer, endian, ())?;
        <T>::write_options(data, writer, endian, ())?;
    } else {
        <u8>::write_options(&0u8, writer, endian, ())?;
    }
    Ok(())
}

#[binrw::binrw]
#[derive(Symbol, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[key(SymbolKey::Message)]
pub enum MessageSymbolType {
    #[brw(magic = 0u8)]
    Single(MessageSymbol),
    #[brw(magic = 1u8)]
    Stream(StreamSymbol),
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
#[binrw::binrw]
#[derive(Symbol, Debug, Clone, Copy, PartialEq, Eq)]
#[key(SymbolKey::StreamPayloadField)]
pub enum StreamPayloadFieldSymbol {
    #[brw(magic = 0u8)]
    Field(FieldSymbol),
    #[brw(magic = 1u8)]
    Array(ArrayFieldSymbol),
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
