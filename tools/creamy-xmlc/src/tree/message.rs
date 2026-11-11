use creamy_protocol_model::Direction;

use super::{
    FieldParser, StreamFieldParser,
    nodes::{ArrayNode, MessageNode, StreamNode},
};
use crate::{
    define_toplevel_parser,
    tokenizer::{Identifier, IdentifierOrArrayOrDynArray},
};

pub struct HeapArrayNode<'src> {
    pub kind: Identifier<'src>,
}

pub enum StreamFieldTypeNode<'src> {
    Type(Identifier<'src>),
    StaticArray(ArrayNode<'src>),
    DynamicArray(HeapArrayNode<'src>),
}

impl<'src> StreamFieldTypeNode<'src> {
    pub fn new(token: IdentifierOrArrayOrDynArray<'src>) -> Self {
        match token {
            IdentifierOrArrayOrDynArray::Identifier(ident) => StreamFieldTypeNode::Type(ident),
            IdentifierOrArrayOrDynArray::Array(ident, size) => {
                StreamFieldTypeNode::StaticArray(ArrayNode { kind: ident, size })
            }
            IdentifierOrArrayOrDynArray::DynArray(ident) => {
                StreamFieldTypeNode::DynamicArray(HeapArrayNode { kind: ident })
            }
        }
    }
}

pub struct StreamFieldNode<'src> {
    pub name: Identifier<'src>,
    pub kind: StreamFieldTypeNode<'src>,
}

define_toplevel_parser! {
    name:       MessageParser,
    type:       MessageNode<'src>,
    top_parse:  parse_single,
    misc_parse: FieldParser::parse_fields,
    args:       [ name: Identifier<'src>, direction: Direction],
    ctor:       |fields, name, direction|  {
        MessageNode { name, fields, direction }
    }
}

define_toplevel_parser! {
    name:       StreamParser,
    type:       StreamNode<'src>,
    top_parse:  parse_stream,
    misc_parse: StreamFieldParser::parse_fields,
    args:       [ name: Identifier<'src>, timeout: u8, direction: Direction],
    ctor:       |fields, name, timeout, direction|  {
        StreamNode { name, direction, timeout, fields }
    }
}
