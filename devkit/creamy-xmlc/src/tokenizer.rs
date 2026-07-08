use std::{cell::RefCell, fmt::Display, ops::Deref, str::FromStr};

use creamy_utils::strpool::{NonZeroStringId, StringId, StringPool};
use miette::SourceSpan;
use roxmltree::{Document, Node, NodeType, TextPos};

use crate::{
    Access, StringPoolIntern, VariantValue, Version,
    diagnostics::Diagnostics,
    error::{Fallback, ProtocolError, ProtocolErrorExt, SyntaxError},
};

impl Fallback for String {
    fn fallback() -> Self {
        ERROR_IDENT.to_string()
    }
}

impl Fallback for &str {
    fn fallback() -> Self {
        ERROR_IDENT
    }
}

impl Fallback for usize {
    fn fallback() -> Self {
        0
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Identifier<'a>(&'a str);
impl Deref for Identifier<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl StringPoolIntern for Identifier<'_> {
    fn intern(&self, pool: &mut StringPool) -> StringId {
        self.0.intern(pool)
    }

    fn intern_non_zero(&self, pool: &mut StringPool) -> NonZeroStringId {
        self.0.intern_non_zero(pool)
    }
}

impl Fallback for Identifier<'_> {
    fn fallback() -> Self {
        Identifier(ERROR_IDENT)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IdentifierOrArray<'src> {
    Identifier(Identifier<'src>),
    Array(&'src str, u8),
}

impl Fallback for IdentifierOrArray<'_> {
    fn fallback() -> Self {
        IdentifierOrArray::Identifier(Identifier::fallback())
    }
}

#[derive(Debug)]
pub struct Number<'a>(&'a str);
impl Fallback for Number<'_> {
    fn fallback() -> Self {
        Self(ERROR_NUMBER)
    }
}

#[derive(Debug)]
pub enum Token<'src> {
    Protocol {
        name: Identifier<'src>,
        version: Version,
        span: SourceSpan,
    },
    Group {
        name: Identifier<'src>,
        access: Access,
        span: SourceSpan,
    },
    Flags {
        name: Identifier<'src>,
        span: SourceSpan,
    },
    Option {
        name: Identifier<'src>,
        span: SourceSpan,
    },
    Bitset {
        name: Identifier<'src>,
        span: SourceSpan,
    },
    BValue {
        name: Identifier<'src>,
        bits: usize,
        repr: Identifier<'src>,
        span: SourceSpan,
    },
    Message {
        kind: u8,
        name: Identifier<'src>,
        span: SourceSpan,
    },
    Struct {
        name: Identifier<'src>,
        span: SourceSpan,
    },
    Enum {
        name: Identifier<'src>,
        repr: Identifier<'src>,
        span: SourceSpan,
    },
    Variant {
        name: Identifier<'src>,
        value: VariantValue,
        span: SourceSpan,
    },
    Field {
        name: Identifier<'src>,
        kind: IdentifierOrArray<'src>,
        span: SourceSpan,
    },
    Array {
        name: Identifier<'src>,
        kind: Identifier<'src>,
        len: Identifier<'src>,
        span: SourceSpan,
    },
    Stream {
        kind: u8,
        name: Identifier<'src>,
        timeout: u8,
        span: SourceSpan,
    },
    StreamStart {
        span: SourceSpan,
    },
    StreamPayload {
        span: SourceSpan,
    },
    StreamEnd {
        span: SourceSpan,
    },
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            Token::Protocol { .. } => "protocol",
            Token::Group { .. } => "group",
            Token::Message { .. } => "message",
            Token::Struct { .. } => "struct",
            Token::Enum { .. } => "enum",
            Token::Variant { .. } => "variant",
            Token::Field { .. } => "field",
            Token::Array { .. } => "remainder",
            Token::Flags { .. } => "flags",
            Token::Option { .. } => "option",
            Token::Bitset { .. } => "bitset",
            Token::BValue { .. } => "value",
            Token::Stream { .. } => "stream",
            Token::StreamStart { .. } => "start",
            Token::StreamPayload { .. } => "payload",
            Token::StreamEnd { .. } => "end",
        };

        write!(f, "<{token}>")
    }
}

const ERROR_IDENT: &str = "Error";
const ERROR_VERSION: &str = "0.0";
const ERROR_ACCESS: &str = "Exclusive";
const ERROR_NUMBER: &str = "1";

fn span_for(node: Node) -> SourceSpan {
    let node_slice = &node.document().input_text()[node.range()];
    let trimmed_slice = node_slice[1..].trim();
    let mut len = 0;
    for c in trimmed_slice.chars() {
        if c.is_whitespace() || c == '/' || c == '>' {
            break;
        }
        len += 1;
    }
    SourceSpan::new((node.range().start + 1).into(), len)
}

pub struct Context<'a, 'src: 'a> {
    //TODO fix
    diagnostics: &'a RefCell<Diagnostics>,
    node: Node<'a, 'src>,
}

impl<'src> Context<'_, 'src> {
    fn tag(&self) -> &'static str {
        match self.node.tag_name().name() {
            "protocol" => "protocol",
            "group" => "group",
            "message" => "message",
            "struct" => "struct",
            "enum" => "enum",
            "field" => "field",
            "variant" => "variant",
            "flags" => "flags",
            "option" => "option",
            "bitset" => "bitset",
            "value" => "value",
            "stream" => "stream",
            "start" => "start",
            "payload" => "payload",
            "end" => "end",
            "array" => "array",
            _ => unreachable!(),
        }
    }

    fn read_attr(&self, attr: &'static str, fallback: &'static str) -> &'src str {
        if let Some(attr) = self.node.attribute_node(attr) {
            &self.node.document().input_text()[attr.range_value()]
        } else {
            self.diagnostics
                .borrow_mut()
                .report_err(SyntaxError::MissingAttribute {
                    tag: self.tag(),
                    attr,
                    span: span_for(self.node),
                });
            fallback
        }
    }

    fn read_ident(&self, attr: &'static str) -> Identifier<'src> {
        self.parse_ident(self.read_attr(attr, ERROR_IDENT), attr, ERROR_IDENT)
    }

    fn read_version(&self) -> Version {
        Version::new(self.read_attr("version", ERROR_VERSION), || {
            self.attribute_value_span("version")
        })
        .or_recover(&mut self.diagnostics.borrow_mut())
    }

    fn attribute_full_span(&self, name: &'static str) -> SourceSpan {
        let attribute = self.node.attribute_node(name).expect("Unreachable!");
        let range = attribute.range();
        SourceSpan::new((range.start).into(), range.len())
    }

    //fn attribute_ident_span(&self, name: &'static str) -> SourceSpan {
    //    let attribute = self.node.attribute_node(name).expect("Unreachable!");
    //    let range = attribute.range_qname();
    //    SourceSpan::new((range.start).into(), range.len())
    //}

    fn attribute_value_span(&self, name: &'static str) -> SourceSpan {
        let attribute = self.node.attribute_node(name).expect("Unreachable!");
        let range = attribute.range_value();
        SourceSpan::new((range.start).into(), range.len())
    }

    fn read_access(&self) -> Access {
        self.parse_access(
            self.parse_ident(
                self.read_attr("access", ERROR_ACCESS),
                "access",
                ERROR_ACCESS,
            ),
            "access",
        )
    }

    fn report_err(&self, error: impl Into<ProtocolError>) {
        self.diagnostics.borrow_mut().report_err(error);
    }

    fn parse_ident(
        &self,
        s: &'src str,
        attr: &'static str,
        fallback: &'static str,
    ) -> Identifier<'src> {
        if s.is_empty() {
            self.report_err(SyntaxError::EmptyIdentifier {
                span: self.attribute_full_span(attr),
            });
            return Identifier(fallback);
        }

        let mut chars = s.chars();

        if let Some(first) = chars.next()
            && !first.is_alphabetic()
            && first != '_'
        {
            self.report_err(SyntaxError::InvalidIdentifier {
                span: self.attribute_value_span(attr),
            });
            return Identifier(fallback);
        }

        if chars.all(|c| c.is_alphanumeric() || c == '_') {
            Identifier(s)
        } else {
            self.report_err(SyntaxError::InvalidIdentifier {
                span: self.attribute_value_span(attr),
            });
            Identifier(fallback)
        }
    }

    fn parse_access(&self, value: Identifier, attr: &'static str) -> Access {
        match value.0 {
            "Public" => Access::Public,
            "Protected" => Access::Protected,
            "Private" => Access::Private,
            "Exclusive" => Access::Exclusive,
            _ => {
                self.diagnostics
                    .borrow_mut()
                    .report_err(SyntaxError::InvalidAccess {
                        span: self.attribute_value_span(attr),
                    });
                Access::Exclusive
            }
        }
    }

    fn parse_number(&self, attr: &'static str) -> Number<'src> {
        let s = self.read_attr(attr, ERROR_NUMBER);
        if s.starts_with('-') && s.chars().skip(1).all(char::is_numeric)
            || s.chars().all(char::is_numeric)
        {
            Number(s)
        } else {
            self.report_err(SyntaxError::NotANumber {
                span: self.attribute_value_span(attr),
            });
            Number::fallback()
        }
    }

    //fn read_num(&self, attr: &'static str, fallback: &'static str) -> Number {
    //    Number::new(self.read_attr(attr, fallback), || {
    //        self.attribute_value_span(attr)
    //    })
    //    .or_recover(&mut self.diagnostics.borrow_mut())
    //}

    fn read_usize(&self, attr: &'static str) -> usize {
        let number = self.parse_number(attr);
        usize::from_str(number.0).expect("Unreachable!")
    }

    fn read_u8(&self, attr: &'static str) -> u8 {
        let number = self.parse_number(attr);
        match u8::from_str(number.0) {
            Ok(value) => value,
            Err(error) => {
                //TODO: custom error
                self.report_err(SyntaxError::IntParse {
                    span: self.attribute_value_span(attr),
                    error,
                });
                0
            }
        }
    }

    fn read_variant_value(&self, attr: &'static str) -> VariantValue {
        let number = self.parse_number(attr);

        if let Ok(value) = number.0.parse::<u64>() {
            VariantValue::Unsigned(value)
        } else if let Ok(value) = number.0.parse::<i64>() {
            VariantValue::Singed(value)
        } else {
            self.report_err(SyntaxError::NotANumber {
                span: self.attribute_value_span(attr),
            });
            VariantValue::Unsigned(1)
        }
    }

    fn try_parse_array(&self, s: &'src str, attr: &'static str) -> Option<IdentifierOrArray<'src>> {
        let s = s.trim();

        let l_bracket = s.starts_with('[');
        let r_bracket = s.ends_with(']');

        if (!l_bracket && r_bracket) || (!r_bracket && l_bracket) {
            self.report_err(SyntaxError::InvalidArraySyntax {
                span: self.attribute_value_span(attr),
            });
            return Some(IdentifierOrArray::Array(ERROR_IDENT, 1));
        } else if !l_bracket && !r_bracket {
            return None;
        }

        let content = &s[1..s.len() - 1];
        let mut parts = content.split(';');
        let (type_ident, count) =
            if let (Some(l), Some(r), None) = (parts.next(), parts.next(), parts.next()) {
                (l.trim(), r.trim())
            } else {
                self.report_err(SyntaxError::InvalidArraySyntax {
                    span: self.attribute_value_span(attr),
                });
                return Some(IdentifierOrArray::Array(ERROR_IDENT, 1));
            };

        Some(match count.parse::<u8>() {
            Ok(count) => IdentifierOrArray::Array(type_ident, count),
            Err(error) => {
                self.report_err(SyntaxError::IntParse {
                    span: self.attribute_value_span(attr),
                    error,
                });
                IdentifierOrArray::Array(ERROR_IDENT, 1)
            }
        })
    }

    fn read_identifier_or_array(&self, attr: &'static str) -> IdentifierOrArray<'src> {
        let s = self.read_attr(attr, ERROR_IDENT);
        if let Some(result) = self.try_parse_array(s, attr) {
            result
        } else {
            IdentifierOrArray::Identifier(self.parse_ident(s, attr, ERROR_IDENT))
        }
    }
}

impl<'a, 'src: 'a> Token<'src> {
    fn new_protocol(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Protocol {
            name: ctx.read_ident("name"),
            version: ctx.read_version(),
            span: span_for(ctx.node),
        }
    }

    fn new_group(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Group {
            name: ctx.read_ident("name"),
            access: ctx.read_access(),
            span: span_for(ctx.node),
        }
    }

    fn new_message(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Message {
            kind: ctx.read_u8("kind"),
            name: ctx.read_ident("name"),
            span: span_for(ctx.node),
        }
    }

    fn new_struct(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Struct {
            name: ctx.read_ident("name"),
            span: span_for(ctx.node),
        }
    }

    fn new_enum(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Enum {
            name: ctx.read_ident("name"),
            repr: ctx.read_ident("repr"),
            span: span_for(ctx.node),
        }
    }

    fn new_field(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Field {
            name: ctx.read_ident("name"),
            kind: ctx.read_identifier_or_array("type"),
            span: span_for(ctx.node),
        }
    }

    fn new_variant(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Variant {
            name: ctx.read_ident("name"),
            value: ctx.read_variant_value("value"),
            span: span_for(ctx.node),
        }
    }

    fn new_flags(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Flags {
            name: ctx.read_ident("name"),
            span: span_for(ctx.node),
        }
    }

    fn new_option(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Option {
            name: ctx.read_ident("name"),
            span: span_for(ctx.node),
        }
    }

    fn new_bitset(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Bitset {
            name: ctx.read_ident("name"),
            span: span_for(ctx.node),
        }
    }

    fn new_value(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::BValue {
            name: ctx.read_ident("name"),
            bits: ctx.read_usize("bits"),
            repr: ctx.read_ident("repr"),
            span: span_for(ctx.node),
        }
    }

    fn new_stream(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Stream {
            kind: ctx.read_u8("kind"),
            name: ctx.read_ident("name"),
            timeout: ctx.read_u8("timeout"),
            span: span_for(ctx.node),
        }
    }

    fn new_array(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::Array {
            name: ctx.read_ident("name"),
            kind: ctx.read_ident("type"),
            len: ctx.read_ident("len_type"),
            span: span_for(ctx.node),
        }
    }

    fn new_stream_start(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::StreamStart {
            span: span_for(ctx.node),
        }
    }

    fn new_stream_payload(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::StreamPayload {
            span: span_for(ctx.node),
        }
    }

    fn new_stream_end(ctx: &Context<'a, 'src>) -> Token<'src> {
        Token::StreamEnd {
            span: span_for(ctx.node),
        }
    }

    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        *match self {
            Token::Protocol {
                name: _,
                version: _,
                span,
            }
            | Token::Group {
                name: _,
                access: _,
                span,
            }
            | Token::Flags { name: _, span }
            | Token::Option { name: _, span }
            | Token::Bitset { name: _, span }
            | Token::BValue {
                name: _,
                bits: _,
                repr: _,
                span,
            }
            | Token::Message {
                kind: _,
                name: _,
                span,
            }
            | Token::Struct { name: _, span }
            | Token::Enum {
                name: _,
                repr: _,
                span,
            }
            | Token::Variant {
                name: _,
                value: _,
                span,
            }
            | Token::Field {
                name: _,
                kind: _,
                span,
            }
            | Token::Array {
                name: _,
                len: _,
                kind: _,
                span,
            }
            | Token::Stream {
                kind: _,
                name: _,
                timeout: _,
                span,
            }
            | Token::StreamStart { span }
            | Token::StreamPayload { span }
            | Token::StreamEnd { span } => span,
        }
    }
}

fn get_content_span(content: &str, pos: TextPos) -> SourceSpan {
    let mut y_pos = 1;
    let mut row_pos = 0;
    let mut idx = 0;
    for char in content.chars() {
        idx += 1;
        if char != '\n' {
            continue;
        }

        row_pos = idx;
        y_pos += 1;
        if y_pos == pos.row {
            break;
        }
    }
    SourceSpan::new(row_pos.into(), pos.col as usize)
}

pub fn tokenize<'a, 'src: 'a>(
    content: &'src str,
    diagnostics: &'a RefCell<Diagnostics>,
) -> Vec<Token<'src>> {
    let mut tokens = vec![];
    let document = match Document::parse(content) {
        Ok(document) => document,
        Err(error) => {
            let at = get_content_span(content, error.pos());
            diagnostics
                .borrow_mut()
                .report_err(SyntaxError::Xml { span: at, error });
            return vec![];
        }
    };

    for node in document
        .root()
        .descendants()
        .filter(|n| n.node_type() == NodeType::Element)
    {
        let ctx = Context { diagnostics, node };
        match node.tag_name().name() {
            "protocol" => tokens.push(Token::new_protocol(&ctx)),
            "group" => tokens.push(Token::new_group(&ctx)),
            "message" => tokens.push(Token::new_message(&ctx)),
            "struct" => tokens.push(Token::new_struct(&ctx)),
            "enum" => tokens.push(Token::new_enum(&ctx)),
            "field" => tokens.push(Token::new_field(&ctx)),
            "variant" => tokens.push(Token::new_variant(&ctx)),
            "flags" => tokens.push(Token::new_flags(&ctx)),
            "option" => tokens.push(Token::new_option(&ctx)),
            "bitset" => tokens.push(Token::new_bitset(&ctx)),
            "value" => tokens.push(Token::new_value(&ctx)),
            "stream" => tokens.push(Token::new_stream(&ctx)),
            "start" => tokens.push(Token::new_stream_start(&ctx)),
            "payload" => tokens.push(Token::new_stream_payload(&ctx)),
            "end" => tokens.push(Token::new_stream_end(&ctx)),
            "array" => tokens.push(Token::new_array(&ctx)),

            _ => diagnostics
                .borrow_mut()
                .report_err(SyntaxError::UnknownTag {
                    span: span_for(node),
                }),
        }
    }

    tokens
}
