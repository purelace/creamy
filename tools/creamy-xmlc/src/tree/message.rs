use creamy_protocol_model::{Direction, FieldsRange, constraints::MAX_FIELDS};
use creamy_utils::strpool::{StringId, StringPool, StringPoolIntern};

use super::nodes::{ArrayNode, MessageNode, MessageNodeType, StreamNode};
use crate::{
    define_node, define_readonly_struct,
    error::AstError,
    tokenizer::{Identifier, IdentifierOrArrayOrDynArray},
    tree::RangeBuilder,
};

define_readonly_struct! {
    [no_brw]
    struct HeapArrayNode {
        kind: StringId,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamFieldTypeNode {
    Type(StringId),
    StaticArray(ArrayNode),
    DynamicArray(HeapArrayNode),
}

impl StreamFieldTypeNode {
    pub fn new(token: IdentifierOrArrayOrDynArray, pool: &mut StringPool) -> Self {
        match token {
            IdentifierOrArrayOrDynArray::Identifier(ident) => {
                StreamFieldTypeNode::Type(pool.get_id_or_add(&ident))
            }
            IdentifierOrArrayOrDynArray::Array(ident, size) => {
                let ident = pool.get_id_or_add(ident);
                StreamFieldTypeNode::StaticArray(ArrayNode::new(ident, size))
            }
            IdentifierOrArrayOrDynArray::DynArray(ident) => {
                let ident = pool.get_id_or_add(ident);
                StreamFieldTypeNode::DynamicArray(HeapArrayNode::new(ident))
            }
        }
    }

    pub const fn type_name(self) -> StringId {
        match self {
            StreamFieldTypeNode::Type(id) => id,
            StreamFieldTypeNode::StaticArray(node) => node.kind(),
            StreamFieldTypeNode::DynamicArray(node) => node.kind,
        }
    }
}

define_node!("stream.field.node", StreamFieldNode, false);
define_readonly_struct! {
    [no_brw]
    [element(MAX_FIELDS, FieldsRange)]
    struct StreamFieldNode {
        name: StringId,
        kind: StreamFieldTypeNode,
    }
}

#[derive(Default)]
pub(crate) struct MessageParser {
    has_error: bool,
    builder: RangeBuilder,
}

impl MessageParser {
    pub fn parse_single(
        &mut self,
        kind: u8,
        name: Identifier,
        direction: Direction,
        storage: &mut crate::tree::storage::NodeStorage,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
        ctx: &mut crate::tree::ParserContext,
    ) {
        let range = ctx.field.parse_fields(ctx.diag, ctx.pool, storage, iter);
        let node = MessageNodeType::Single(MessageNode::new(
            name.intern(ctx.pool),
            range,
            direction,
            kind,
        ));
        self.builder.next();
        crate::push_node!(to: storage, node: node,flag: self.has_error,ctx.diag,AstError::TooManyMessages);
    }

    pub const fn builder(&mut self) -> &mut crate::tree::RangeBuilder {
        &mut self.builder
    }

    pub fn parse_stream(
        &mut self,
        kind: u8,
        name: Identifier,
        direction: Direction,
        timeout: u8,
        storage: &mut crate::tree::storage::NodeStorage,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
        ctx: &mut crate::tree::ParserContext,
    ) {
        let fields = ctx
            .stream_field
            .parse_fields(ctx.diag, ctx.pool, storage, iter);

        let node = MessageNodeType::Stream(StreamNode::new(
            name.intern(ctx.pool),
            direction,
            timeout,
            kind,
            fields,
        ));

        self.builder.next();
        crate::push_node!(to: storage, node: node, flag: self.has_error, ctx.diag, AstError::TooManyMessages);
    }
}
