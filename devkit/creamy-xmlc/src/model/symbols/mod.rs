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
pub use repr::EnumRepr;

use crate::{
    Access,
    constraints::{
        MAX_BITSET_VALUES, MAX_BITSETS, MAX_FIELDS, MAX_FLAGS, MAX_GROUPS, MAX_MESSAGES,
        MAX_OPTIONS, MAX_STRUCTS,
    },
    define_readonly_struct,
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
            Type::Struct(symbol) => symbol.name(),
            Type::Enum(symbol) => symbol.name(),
            Type::Flags(symbol) => symbol.name(),
            Type::Bitset(symbol) => symbol.name(),
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
        name: StringId,
        fields: FieldsRange,
    }
}

define_readonly_struct! {
    [element(MAX_GROUPS, GroupsRange)]
    struct GroupSymbol {
        name: StringId,
        access: Access,
        messages: MessagesRange,
        types: TypesRange,
    }
}

define_readonly_struct! {
    [element(MAX_MESSAGES, MessagesRange)]
    struct MessageSymbol {
        name: StringId,
        fields: FieldsRange,
        kind: u8,
    }
}

define_readonly_struct! {
    [element(MAX_MESSAGES, MessagesRange)]
    struct StreamSymbol {
        [Documentation("Name of message.")]
        name: StringId,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub enum MessageSymbolType {
    Single(MessageSymbol),
    Stream(StreamSymbol),
}

impl VectorElement for MessageSymbolType {
    const MAX_SIZE: usize = MAX_MESSAGES;
    type RangeType = MessagesRange;
}

impl MessageSymbolType {
    #[must_use]
    pub const fn name(&self) -> StringId {
        match self {
            MessageSymbolType::Single(m) => m.name(),
            MessageSymbolType::Stream(s) => s.name(),
        }
    }
}

define_readonly_struct! {
    [element(MAX_FLAGS, FlagsRange)]
    struct FlagsSymbol {
        name: StringId,
        values: OptionsRange,
    }
}

define_readonly_struct! {
    [element(MAX_BITSETS, BitsetsRange)]
    struct BitsetSymbol {
        name: StringId,
        values: BValuesRange,
    }
}

define_readonly_struct! {
    [element(MAX_OPTIONS, OptionsRange)]
    struct OptionSymbol {
        name: StringId,
    }
}

define_readonly_struct! {
    [element(MAX_BITSET_VALUES, BValuesRange)]
    struct BitsetValueSymbol {
        name: StringId,
        repr: TypeId,
        bits: u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead, BinWrite)]
pub enum StreamPayloadFieldSymbol {
    Field(FieldSymbol),
    Array(ArrayFieldSymbol),
}

impl VectorElement for StreamPayloadFieldSymbol {
    const MAX_SIZE: usize = MAX_FIELDS;
    type RangeType = FieldsRange;
}

define_readonly_struct! {
    struct ArrayFieldSymbol{
        name: StringId,
        kind: TypeId,
        len: StringId,
    }
}
