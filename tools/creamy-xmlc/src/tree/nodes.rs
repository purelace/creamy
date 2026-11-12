use creamy_protocol_model::{Direction, symbols::VariantValue};

use super::StreamFieldNode;
use crate::tokenizer::Identifier;

pub struct EnumNode<'src> {
    pub name: Identifier<'src>,
    pub repr: Identifier<'src>,
    pub variants: Vec<VariantNode<'src>>,
}

pub struct VariantNode<'src> {
    pub name: Identifier<'src>,
    pub value: VariantValue,
}

pub enum MessageNodeType<'src> {
    Single(MessageNode<'src>),
    Stream(StreamNode<'src>),
}

impl MessageNodeType<'_> {
    pub fn name(&self) -> &str {
        match self {
            MessageNodeType::Single(m) => &m.name,
            MessageNodeType::Stream(s) => &s.name,
        }
    }
}

pub struct MessageNode<'src> {
    pub name: Identifier<'src>,
    pub fields: Vec<FieldNode<'src>>,
    pub direction: Direction,
}

pub struct StructNode<'src> {
    pub name: Identifier<'src>,
    pub fields: Vec<FieldNode<'src>>,
}

#[derive(Debug)]
pub struct ArrayNode<'src> {
    pub kind: Identifier<'src>,
    pub size: u8,
}

#[derive(Debug)]
pub enum FieldTypeNode<'src> {
    Type(Identifier<'src>),
    Array(ArrayNode<'src>),
}

impl FieldTypeNode<'_> {
    pub fn type_name(&self) -> &str {
        match self {
            FieldTypeNode::Type(id) => id,
            FieldTypeNode::Array(node) => &node.kind,
        }
    }
}

pub struct FieldNode<'src> {
    pub name: Identifier<'src>,
    pub kind: FieldTypeNode<'src>,
}

pub struct FlagsNode<'src> {
    pub name: Identifier<'src>,
    pub options: Vec<OptionNode<'src>>,
}

#[derive(Copy, Clone)]
pub struct OptionNode<'src> {
    pub name: Identifier<'src>,
}

pub struct BitsetNode<'src> {
    pub name: Identifier<'src>,
    pub values: Vec<BitsetValueNode<'src>>,
}

pub struct BitsetValueNode<'src> {
    pub name: Identifier<'src>,
    pub repr: Identifier<'src>,
    pub bits: usize,
}

pub struct StreamNode<'src> {
    pub name: Identifier<'src>,
    pub direction: Direction,
    pub timeout: u8,
    pub fields: Vec<StreamFieldNode<'src>>,
}
