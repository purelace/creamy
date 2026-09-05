use super::nodes::{
    FieldNode, FieldTypeNode, MessageNode, MessageNodeType, StreamNode, StreamPayloadFieldNode,
};
use crate::{
    StringPoolIntern,
    error::AstError,
    model::{definition::Direction, symbols::U8_ID},
    tokenizer::{Identifier, Token},
    tree::RangeBuilder,
    utils::FieldsRange,
};

#[inline]
const fn token_stream_payload_field(t: &Token) -> bool {
    matches!(t, Token::Field { .. })
}

#[derive(Default)]
pub(crate) struct StreamPayloadFieldParser {
    //vec: crate::utils::BoundedVec<StreamPayloadFieldNode>,
    has_error: bool,
    builder: crate::tree::RangeBuilder,
}

impl StreamPayloadFieldParser {
    #[allow(unused)]
    pub fn parse_fields(
        &mut self,
        diag: &mut crate::Diagnostics,
        pool: &mut crate::utils::strpool::StringPool,
        storage: &mut crate::tree::storage::NodeStorage,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
    ) -> FieldsRange {
        //TODO: ебаный костыль, читаем ниже.
        let node = StreamPayloadFieldNode::Field(FieldNode::new(
            pool.get_id_or_add("__unused"),
            FieldTypeNode::Type(U8_ID),
        ));
        crate::push_node! {
            to: storage, node: node, flag: self.has_error, diag, AstError::TooManyFields
        };
        self.builder.next();
        while let Some(token) = iter.next_if(token_stream_payload_field) {
            let node = match token {
                Token::Field {
                    name,
                    kind,
                    span: _,
                } => StreamPayloadFieldNode::Field(FieldNode::new(
                    name.intern(pool),
                    FieldTypeNode::new(kind, pool),
                )),
                _ => unreachable!(),
            };

            crate::push_node! {
                to: storage,node: node,flag: self.has_error, diag, AstError::TooManyFields
            }
            self.builder.next();
        }
        self.builder.build_fields()
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
        let mut head = None;
        let mut payload = None;
        let mut tail = None;
        //let mut result = None;

        //Fix: ебаный костыль. надо сделать внедрение полей или на уровне семантической модели или хз
        for i in 0..3 {
            match iter.peek() {
                Some(Token::StreamHead { .. }) if head.is_none() => {
                    let _ = iter.next();
                    assert!(
                        storage.add_node(FieldNode::new(
                            ctx.pool.get_id_or_add("__unused"),
                            FieldTypeNode::Type(U8_ID),
                        )),
                        "исправить ебучий костыль"
                    );
                    ctx.field.builder.next();
                    head = Some(ctx.field.parse_fields(ctx.diag, ctx.pool, storage, iter));
                }
                Some(Token::StreamPayload { .. }) if payload.is_none() => {
                    let _ = iter.next();
                    payload = Some(ctx.payload.parse_fields(ctx.diag, ctx.pool, storage, iter));
                }
                Some(Token::StreamTail { .. }) if tail.is_none() => {
                    let _ = iter.next();
                    assert!(
                        storage.add_node(FieldNode::new(
                            ctx.pool.get_id_or_add("__unused"),
                            FieldTypeNode::Type(U8_ID),
                        )),
                        "исправить ебучий костыль"
                    );
                    ctx.field.builder.next();
                    tail = Some(ctx.field.parse_fields(ctx.diag, ctx.pool, storage, iter));
                }
                //Some(Token::StreamResult { .. }) if result.is_none() => {
                //    let _ = iter.next();
                //    ctx.field.builder.next();
                //}
                _ => {
                    if payload.is_some() {
                        break;
                    }
                    return;
                }
            }
        }

        let Some(payload) = payload else {
            unreachable!("todo");
        };

        let node = MessageNodeType::Stream(StreamNode::new(
            name.intern(ctx.pool),
            direction,
            timeout,
            kind,
            head,
            payload,
            tail,
        ));

        self.builder.next();
        crate::push_node!(to: storage,node: node,flag: self.has_error,ctx.diag,AstError::TooManyMessages);
    }
}
