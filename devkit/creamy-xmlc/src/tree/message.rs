use crate::{
    StringPoolIntern,
    error::AstError,
    model::symbols::U8_ID,
    nodes::{
        ArrayFieldNode, FieldNode, FieldTypeNode, MessageNode, MessageNodeType, StreamNode,
        StreamPayloadFieldNode,
    },
    tokenizer::{Identifier, Token},
    tree::RangeBuilder,
    utils::{BoundedVec, FieldsRange},
};

#[inline]
const fn token_stream_payload_field(t: &Token) -> bool {
    matches!(t, Token::Field { .. } | Token::Array { .. })
}

#[derive(Default)]
pub(crate) struct StreamPayloadFieldParser {
    vec: crate::utils::BoundedVec<StreamPayloadFieldNode>,
    has_error: bool,
    builder: crate::tree::RangeBuilder,
}

impl StreamPayloadFieldParser {
    #[allow(unused)]
    pub fn parse_fields(
        &mut self,
        diag: &mut crate::Diagnostics,
        pool: &mut crate::utils::strpool::StringPool,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
    ) -> FieldsRange {
        //TODO: ебаный костыль, читаем ниже.
        let node = StreamPayloadFieldNode::Field(FieldNode::new(
            pool.get_id_or_add("__unused"),
            FieldTypeNode::Type(U8_ID),
        ));
        crate::push_node! {
            to: self.vec,node: node,flag: self.has_error,diag,AstError::TooManyFields
        };
        self.builder.next();
        while let Some(token) = iter.next_if(token_stream_payload_field) {
            let node = match token {
                Token::Array {
                    name,
                    kind,
                    len,
                    span: _,
                } => StreamPayloadFieldNode::Array(ArrayFieldNode::new(
                    name.intern(pool),
                    kind.intern(pool),
                    len.intern(pool),
                )),
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
                to: self.vec,node: node,flag: self.has_error,diag,AstError::TooManyFields
            }
            self.builder.next();
        }
        self.builder.build_fields()
    }

    pub fn take(self) -> crate::utils::BoundedVec<StreamPayloadFieldNode> {
        self.vec
    }
}

#[derive(Default)]
pub(crate) struct MessageParser {
    vec: BoundedVec<MessageNodeType>,
    has_error: bool,
    builder: RangeBuilder,
}

impl MessageParser {
    pub fn parse_single(
        &mut self,
        kind: u8,
        name: Identifier,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
        ctx: &mut crate::tree::ParserContext,
    ) {
        let range = ctx.field.parse_fields(ctx.diag, ctx.pool, iter);
        let node = MessageNodeType::Single(MessageNode::new(name.intern(ctx.pool), range, kind));
        self.builder.next();
        crate::push_node!(to: self.vec,node: node,flag: self.has_error,ctx.diag,AstError::TooManyMessages);
    }

    pub const fn builder(&mut self) -> &mut crate::tree::RangeBuilder {
        &mut self.builder
    }

    pub fn take(self) -> BoundedVec<MessageNodeType> {
        self.vec
    }

    pub fn parse_stream(
        &mut self,
        kind: u8,
        name: Identifier,
        timeout: u8,
        iter: &mut core::iter::Peekable<std::vec::Drain<crate::tokenizer::Token>>,
        ctx: &mut crate::tree::ParserContext,
    ) {
        let mut start = None;
        let mut payload = None;
        let mut end = None;

        //Fix: ебаный костыль. надо сделать внедрение полей или на уровне семантической модели или хз
        for i in 0..3 {
            match iter.peek() {
                Some(Token::StreamStart { .. }) if start.is_none() => {
                    let _ = iter.next();
                    let range_start = ctx.field.vec.len();
                    assert!(
                        ctx.field.vec.push(FieldNode::new(
                            ctx.pool.get_id_or_add("__unused"),
                            FieldTypeNode::Type(U8_ID),
                        )),
                        "исправить ебучий костыль"
                    );
                    let range = ctx.field.parse_fields(ctx.diag, ctx.pool, iter);
                    start = Some(FieldsRange::new(range_start as u16, range.len() + 1));
                }
                Some(Token::StreamPayload { .. }) if payload.is_none() => {
                    let _ = iter.next();
                    payload = Some(ctx.payload.parse_fields(ctx.diag, ctx.pool, iter));
                }
                Some(Token::StreamEnd { .. }) if end.is_none() => {
                    let _ = iter.next();
                    let range_start = ctx.field.vec.len();
                    assert!(
                        ctx.field.vec.push(FieldNode::new(
                            ctx.pool.get_id_or_add("__unused"),
                            FieldTypeNode::Type(U8_ID),
                        )),
                        "исправить ебучий костыль"
                    );
                    let range = ctx.field.parse_fields(ctx.diag, ctx.pool, iter);
                    end = Some(FieldsRange::new(range_start as u16, range.len() + 1));
                }
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
            timeout,
            kind,
            start,
            payload,
            end,
        ));

        self.builder.next();
        crate::push_node!(to: self.vec,node: node,flag: self.has_error,ctx.diag,AstError::TooManyMessages);
    }
}
