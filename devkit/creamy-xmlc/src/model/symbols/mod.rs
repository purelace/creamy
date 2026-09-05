mod enumeration;
mod field;
mod numeric;
mod remainder;
mod repr;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;
pub use enumeration::{EnumSymbol, VariantSymbol};
pub use field::{FieldSymbol, FieldType};
pub use numeric::*;
pub use remainder::Remainder;
pub use repr::PrimitiveRepr;

use super::{
    definition::Direction,
    storage::{Symbol, SymbolKey},
};
use crate::{
    Access,
    constraints::{
        MAX_BITSET_VALUES, MAX_BITSETS, MAX_FIELDS, MAX_FLAGS, MAX_GROUPS, MAX_MESSAGES,
        MAX_OPTIONS, MAX_STRUCTS,
    },
    define_readonly_struct, impl_with_ident,
    table::TypeId,
    utils::{
        BitsetValuesRange, BitsetsRange, FieldsRange, FlagsRange, GroupsRange, MessagesRange,
        OptionsRange, Size, StructsRange, TypesRange, VectorElement,
    },
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[brw(magic = 0u8)]
    Numeric(NumericSymbol),
    #[brw(magic = 1u8)]
    Array(ArraySymbol),
    #[brw(magic = 2u8)]
    Struct(StructSymbol),
    #[brw(magic = 3u8)]
    Enum(EnumSymbol),
    #[brw(magic = 4u8)]
    Flags(FlagsSymbol),
    #[brw(magic = 5u8)]
    Bitset(BitsetSymbol),
}

impl Type {
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use]
    pub const fn ident(&self) -> StringId {
        match self {
            Type::Numeric(symbol) => symbol.name(),
            Type::Array(_) => unreachable!(),
            Type::Struct(symbol) => symbol.ident(),
            Type::Enum(symbol) => symbol.ident(),
            Type::Flags(symbol) => symbol.ident(),
            Type::Bitset(symbol) => symbol.ident(),
        }
    }

    #[must_use]
    pub const fn with_ident(&self, id: StringId) -> Self {
        match self {
            Type::Numeric(s) => Type::Numeric(*s),
            Type::Array(s) => Type::Array(*s),
            Type::Struct(s) => Type::Struct(s.with_ident(id)),
            Type::Enum(s) => Type::Enum(s.with_ident(id)),
            Type::Flags(s) => Type::Flags(s.with_ident(id)),
            Type::Bitset(s) => Type::Bitset(s.with_ident(id)),
        }
    }
}

define_readonly_struct! {
    struct ArraySymbol {
        kind: TypeId,
        len: Size,
    }
}

define_readonly_struct! {
    [element(MAX_STRUCTS, StructsRange)]
    struct StructSymbol {
        ident: StringId,
        fields: FieldsRange,
    }
}
impl_with_ident!(StructSymbol);

define_readonly_struct! {
    struct GlobalTypesSymbol {
        types: TypesRange,
    }
}

define_readonly_struct! {
    [element(MAX_GROUPS, GroupsRange)]
    struct GroupSymbol {
        ident: StringId,
        access: Access,
        messages: MessagesRange,
        types: TypesRange,
    }
}
impl_with_ident!(GroupSymbol);

impl Symbol for GroupSymbol {
    const KEY: SymbolKey = SymbolKey::Group;
}

define_readonly_struct! {
    [element(MAX_MESSAGES, MessagesRange)]
    struct MessageSymbol {
        ident: StringId,
        fields: FieldsRange,
        direction: Direction,
        kind: u8,
    }
}
impl_with_ident!(MessageSymbol);

#[derive(binrw::BinWrite, binrw::BinRead, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamSymbol {
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
crate::define_readonly_struct!(@impl_vector_element StreamSymbol MAX_MESSAGES MessagesRange);
crate::define_readonly_struct!(@impl_methods StreamSymbol {
    ident: StringId,
    direction: Direction,
    timeout: u8,
    kind: u8,
    head: Option<FieldsRange>,
    payload: FieldsRange,
    tail: Option<FieldsRange>,
});

fn read_opt<T: BinRead<Args<'static> = ()>, R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    endian: binrw::Endian,
    _: (),
) -> binrw::BinResult<Option<T>> {
    let has_pos: u8 = <u8>::read_options(reader, endian, ())?;
    if has_pos != 0 {
        let data = <T>::read_options(reader, endian, ())?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

#[allow(clippy::ref_option)]
fn write_opt<T: BinWrite<Args<'static> = ()>, W: std::io::Write + std::io::Seek>(
    opt: &Option<T>,
    writer: &mut W,
    endian: binrw::Endian,
    _: (),
) -> binrw::BinResult<()> {
    if let Some(data) = opt {
        <u8>::write_options(&1u8, writer, endian, ())?;
        <T>::write_options(data, writer, endian, ())?;
    } else {
        <u8>::write_options(&0u8, writer, endian, ())?;
    }
    Ok(())
}

impl_with_ident!(StreamSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite, Hash)]
pub enum MessageSymbolType {
    #[brw(magic = 0u8)]
    Single(MessageSymbol),
    #[brw(magic = 1u8)]
    Stream(StreamSymbol),
}

impl Symbol for MessageSymbolType {
    const KEY: SymbolKey = SymbolKey::Message;
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

define_readonly_struct! {
    [element(MAX_FLAGS, FlagsRange)]
    struct FlagsSymbol {
        ident: StringId,
        values: OptionsRange,
    }
}
impl_with_ident!(FlagsSymbol);

define_readonly_struct! {
    [element(MAX_BITSETS, BitsetsRange)]
    struct BitsetSymbol {
        ident: StringId,
        values: BitsetValuesRange,
    }
}
impl_with_ident!(BitsetSymbol);

define_readonly_struct! {
    [element(MAX_OPTIONS, OptionsRange)]
    struct OptionSymbol {
        ident: StringId,
    }
}
impl_with_ident!(OptionSymbol);

impl Symbol for OptionSymbol {
    const KEY: SymbolKey = SymbolKey::Option;
}

define_readonly_struct! {
    [element(MAX_BITSET_VALUES, BitsetValuesRange)]
    struct BitsetValueSymbol {
        ident: StringId,
        repr: TypeId,
        bits: u8,
    }
}

impl Symbol for BitsetValueSymbol {
    const KEY: SymbolKey = SymbolKey::BitsetValue;
}

impl_with_ident!(BitsetValueSymbol);

// TODO:
// Тут надо пересчитать максимальное количество типов.
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub enum StreamPayloadFieldSymbol {
    #[brw(magic = 0u8)]
    Field(FieldSymbol),
    #[brw(magic = 1u8)]
    Array(ArrayFieldSymbol),
}

impl Symbol for StreamPayloadFieldSymbol {
    const KEY: SymbolKey = SymbolKey::StreamPayloadField;
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

define_readonly_struct! {
    struct ArrayFieldSymbol{
        ident: StringId,
        kind: TypeId,
        len: StringId,
    }
}

impl_with_ident!(ArrayFieldSymbol);
