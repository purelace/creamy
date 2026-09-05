use std::fmt::Display;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::{StringId, StringPool};

use crate::{
    Access,
    constraints::{
        MAX_BITSET_VALUES, MAX_BITSETS, MAX_ENUMS, MAX_FIELDS, MAX_FLAGS, MAX_GROUPS, MAX_MESSAGES,
        MAX_OPTIONS, MAX_STRUCTS, MAX_VARIANTS,
    },
    define_readonly_struct,
    error::Fallback,
    model::definition::Direction,
    tokenizer::IdentifierOrArray,
    utils::{
        BitsetValuesRange, BitsetsRange, EnumsRange, FieldsRange, FlagsRange, GroupsRange,
        MessagesRange, OptionsRange, StreamsFieldsRange, StructsRange, VariantsRange,
        VectorElement,
    },
};

macro_rules! define_node {
    ($key:expr, $name:ident, $is_type: expr) => {
        impl $crate::tree::storage::Node for $name {
            const KEY: $crate::tree::storage::NodeKey = $crate::tree::storage::NodeKey::new($key);
            const IS_TYPE: bool = $is_type;
        }
    };
}

define_node!("enum.node", EnumNode, true);
define_readonly_struct! {
    [no_brw]
    [element(MAX_ENUMS, EnumsRange)]
    struct EnumNode {
        name: StringId,
        repr: StringId,
        variants: VariantsRange,
    }
}

define_node!("enum.variant.node", VariantNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_VARIANTS, VariantsRange)]
    struct VariantNode {
        name: StringId,
        value: VariantValue,
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantValue {
    #[brw(magic = 0u8)]
    Singed(i64),
    #[brw(magic = 1u8)]
    Unsigned(u64),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for VariantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantValue::Singed(s) => write!(f, "{s}"),
            VariantValue::Unsigned(u) => write!(f, "{u}"),
        }
    }
}

impl Fallback for VariantValue {
    fn fallback() -> Self {
        Self::Unsigned(1)
    }
}

define_readonly_struct! {
    [no_brw]
    struct GlobalTypesNode {
        structs: StructsRange,
        enums: EnumsRange,
        flags: FlagsRange,
        bitsets: BitsetsRange,
    }
}

impl Default for GlobalTypesNode {
    fn default() -> Self {
        Self {
            structs: StructsRange::new(0, 0),
            enums: EnumsRange::new(0, 0),
            flags: FlagsRange::new(0, 0),
            bitsets: BitsetsRange::new(0, 0),
        }
    }
}

define_node!("group.node", GroupNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_GROUPS, GroupsRange)]
    struct GroupNode {
        name: StringId,
        access: Access,
        messages: MessagesRange,
        structs: StructsRange,
        enums: EnumsRange,
        flags: FlagsRange,
        bitsets: BitsetsRange,
    }
}

define_node!("message.node.type", MessageNodeType, false);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageNodeType {
    Single(MessageNode),
    Stream(StreamNode),
}

impl VectorElement for MessageNodeType {
    const MAX_SIZE: usize = MAX_MESSAGES;
    type RangeType = MessagesRange;
}

impl MessageNodeType {
    pub const fn name(&self) -> StringId {
        match self {
            MessageNodeType::Single(m) => m.name(),
            MessageNodeType::Stream(s) => s.name(),
        }
    }
}

//define_node!("message.node", MessageNode, false);
define_readonly_struct! {
    [no_brw]
    struct MessageNode {
        name: StringId,
        fields: FieldsRange,
        direction: Direction,
        kind: u8,
    }
}

define_node!("struct.node", StructNode, true);
define_readonly_struct! {
    [no_brw]
    [element(MAX_STRUCTS, StructsRange)]
    struct StructNode {
        name: StringId,
        fields: FieldsRange,
    }
}

//define_node!("array.node", ArrayNode, false);
define_readonly_struct! {
    [no_brw]
    struct ArrayNode {
        kind: StringId,
        size: u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldTypeNode {
    Type(StringId),
    Array(ArrayNode),
}

impl FieldTypeNode {
    pub fn new(token: IdentifierOrArray, pool: &mut StringPool) -> Self {
        match token {
            IdentifierOrArray::Identifier(ident) => FieldTypeNode::Type(pool.get_id_or_add(&ident)),
            IdentifierOrArray::Array(ident, size) => {
                let ident = pool.get_id_or_add(ident);
                FieldTypeNode::Array(ArrayNode::new(ident, size))
            }
        }
    }

    pub const fn type_name(self) -> StringId {
        match self {
            FieldTypeNode::Type(id) => id,
            FieldTypeNode::Array(node) => node.kind,
        }
    }
}

define_node!("field.node", FieldNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_FIELDS, FieldsRange)]
    struct FieldNode {
        name: StringId,
        kind: FieldTypeNode,
    }
}

define_node!("flags.node", FlagsNode, true);
define_readonly_struct! {
    [no_brw]
    [element(MAX_FLAGS, FlagsRange)]
    struct FlagsNode {
        ident: StringId,
        options: OptionsRange,
    }
}

define_node!("option.node", OptionNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_OPTIONS, OptionsRange)]
    struct OptionNode {
        ident: StringId,
    }
}

define_node!("bitset.node", BitsetNode, true);
define_readonly_struct! {
    [no_brw]
    [element(MAX_BITSETS, BitsetsRange)]
    struct BitsetNode {
        ident: StringId,
        values: BitsetValuesRange,
    }
}

define_node!("bitset.value.node", BitsetValueNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_BITSET_VALUES, BitsetValuesRange)]
    struct BitsetValueNode {
        ident: StringId,
        repr: StringId,
        bits: usize,
    }
}

//define_node!("stream.node", StreamNode, false);
define_readonly_struct! {
    [no_brw]
    struct StreamNode {
        name: StringId,
        direction: Direction,
        timeout: u8,
        kind: u8,
        head: Option<FieldsRange>,
        payload: FieldsRange,
        tail: Option<FieldsRange>,
    }
}

define_node!("stream.payload.node", StreamPayloadFieldNode, false);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamPayloadFieldNode {
    Field(FieldNode),
    //Array(ArrayFieldNode),
}

impl VectorElement for StreamPayloadFieldNode {
    const MAX_SIZE: usize = MAX_FIELDS;
    type RangeType = StreamsFieldsRange;
}

//define_node!("array.field.node", ArrayFieldNode, false);
//define_readonly_struct! {
//    [no_brw]
//    struct ArrayFieldNode {
//        name: StringId,
//        kind: StringId,
//        len: StringId,
//    }
//}
