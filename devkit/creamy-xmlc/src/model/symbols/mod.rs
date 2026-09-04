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

use crate::{
    Access,
    constraints::{
        MAX_BITSET_VALUES, MAX_BITSETS, MAX_FIELDS, MAX_FLAGS, MAX_GROUPS, MAX_MESSAGES,
        MAX_OPTIONS, MAX_STRUCTS,
    },
    define_readonly_struct, impl_with_ident,
    table::TypeId,
    utils::{
        BValuesRange, BitsetsRange, FieldsRange, FlagsRange, GroupsRange, MessagesRange,
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
    [element(MAX_GROUPS, GroupsRange)]
    struct GroupSymbol {
        ident: StringId,
        access: Access,
        messages: MessagesRange,
        types: TypesRange,
    }
}
impl_with_ident!(GroupSymbol);

define_readonly_struct! {
    [element(MAX_MESSAGES, MessagesRange)]
    struct MessageSymbol {
        ident: StringId,
        fields: FieldsRange,
        kind: u8,
    }
}
impl_with_ident!(MessageSymbol);

define_readonly_struct! {
    [element(MAX_MESSAGES, MessagesRange)]
    struct StreamSymbol {
        [Documentation("Name of message.")]
        ident: StringId,
        [Documentation("Timeout in frames.")]
        timeout: u8,
        [Documentation("
            Type of the message stream.
            Allowed types: `[
                Head, Payload, Tail
            ]`"
        )]
        kind: u8,
        [Documentation("Head of the message stream.")]
        head: Option<FieldsRange>,
        [Documentation("Payload of the message stream.")]
        payload: FieldsRange,
        [Documentation("Tail of the message stream.")]
        tail: Option<FieldsRange>,
    }
}

impl_with_ident!(StreamSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite, Hash)]
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
            MessageSymbolType::Single(s) => s.kind,
            MessageSymbolType::Stream(s) => s.kind,
        }
    }

    #[must_use]
    pub const fn with_ident(&self, id: StringId) -> Self {
        match self {
            MessageSymbolType::Single(s) => MessageSymbolType::Single(s.with_ident(id)),
            MessageSymbolType::Stream(s) => MessageSymbolType::Stream(s.with_ident(id)),
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
        values: BValuesRange,
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

define_readonly_struct! {
    [element(MAX_BITSET_VALUES, BValuesRange)]
    struct BitsetValueSymbol {
        ident: StringId,
        repr: TypeId,
        bits: u8,
    }
}

impl_with_ident!(BitsetValueSymbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub enum StreamPayloadFieldSymbol {
    Field(FieldSymbol),
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

define_readonly_struct! {
    struct ArrayFieldSymbol{
        ident: StringId,
        kind: TypeId,
        len: StringId,
    }
}

impl_with_ident!(ArrayFieldSymbol);
