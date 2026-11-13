mod bitset;
mod enums;
mod flags;
mod message;
pub mod nodes;
mod structs;

use creamy_protocol_model::{Access, Version, error::Fallback};
pub use message::{StreamFieldNode, StreamFieldTypeNode};

use self::{
    flags::FlagsParser,
    message::{MessageParser, StreamParser},
    nodes::{
        ArrayNode, BitsetNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, MessageNodeType,
        StructNode,
    },
};
use crate::{
    Diagnostics,
    error::AstError,
    tokenizer::{Identifier, IdentifierOrArray, Token},
    tree::{bitset::BitsetParser, enums::EnumParser, structs::StructParser},
};

pub struct ProtocolTree<'src> {
    pub name: Identifier<'src>,
    pub version: Version,
    pub access: Access,
    pub structs: Vec<StructNode<'src>>,
    pub enums: Vec<EnumNode<'src>>,
    pub flags: Vec<FlagsNode<'src>>,
    pub bitsets: Vec<BitsetNode<'src>>,
    pub messages: Vec<MessageNodeType<'src>>,
}

impl<'src> ProtocolTree<'src> {
    pub fn new(mut tokens: Vec<Token<'src>>, diagnostics: &mut Diagnostics) -> Self {
        let mut iter = tokens.drain(..).peekable();

        let (name, version, access) = if let Some(Token::Protocol {
            name,
            version,
            access,
            ..
        }) = iter.next()
        {
            (name, version, access)
        } else {
            diagnostics.report_err(AstError::MissingProtocolToken);
            (
                Identifier::fallback(),
                Version::fallback(),
                Access::fallback(),
            )
        };

        let mut tree = ProtocolTree {
            name,
            version,
            access,
            structs: vec![],
            enums: vec![],
            flags: vec![],
            bitsets: vec![],
            messages: vec![],
        };

        while let Some(token) = iter.next() {
            match token {
                Token::Struct { name, .. } => {
                    tree.structs
                        .push(StructParser::parse_struct(name, &mut iter));
                }
                Token::Enum { name, repr, .. } => {
                    tree.enums
                        .push(EnumParser::parse_enum(name, repr, &mut iter));
                }
                Token::Flags { name, .. } => {
                    tree.flags.push(FlagsParser::parse_flags(name, &mut iter));
                }
                Token::Bitset { name, .. } => {
                    tree.bitsets
                        .push(BitsetParser::parse_bitset(name, &mut iter));
                }
                Token::Message {
                    name, direction, ..
                } => {
                    tree.messages
                        .push(MessageNodeType::Single(MessageParser::parse_single(
                            name, direction, &mut iter,
                        )));
                }
                Token::Stream {
                    name,
                    timeout,
                    direction,
                    ..
                } => {
                    tree.messages
                        .push(MessageNodeType::Stream(StreamParser::parse_stream(
                            name, timeout, direction, &mut iter,
                        )));
                }
                other => {
                    let span = other.span();
                    diagnostics.report_err(AstError::UnexpectedToken { span });
                }
            }
        }

        tree
    }

    /// Значение не может превышать [``crate::constraints::MAX_TYPE_COUNT``]
    #[allow(clippy::cast_possible_truncation)]
    pub fn type_count(&self) -> u16 {
        (self.structs.len() + self.enums.len() + self.flags.len() + self.bitsets.len()) as u16
    }
}

#[macro_export]
macro_rules! define_misc_parser {
    (
        name:    $name: ident,
        type:    $type: ty,
        fn:      $parse_all: ident,
        token:   $token_path: path,
        fields:  [ $($f_name:ident),* $(,)? ],
        predicate: $predicate: expr,
        ctor:    |$($c_arg:ident),*| $closure:expr
    ) => {
        pub(crate) struct $name;
        impl $name {
            #[allow(unused)]
            pub fn $parse_all<'src>(
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token<'src>>>,
            ) -> Vec<$type> {
                let mut vector = Vec::new();
                while let Some($token_path { $($f_name,)* .. }) = iter.next_if($predicate) {
                    let node = {
                        $(let $c_arg = $f_name;)*
                        $closure
                    };

                    vector.push(node);
                }
                vector
            }
        }
    };
}

#[macro_export]
macro_rules! define_toplevel_parser {
    (
        name:       $name:ident,
        type:       $type:ty,
        top_parse:  $t_parse_method:ident,
        misc_parse: $misc:ident::$m_parse:ident,
        args:       [ $($arg:ident : $arg_type:ty),* $(,)? ],
        ctor:       |$nodes:ident, $($c_arg:ident),*| $closure:expr
    ) => {
        pub(crate) struct $name;

        impl $name {
            pub fn $t_parse_method<'src>(
                $($arg : $arg_type,)*
                iter: &mut core::iter::Peekable<std::vec::Drain<$crate::tokenizer::Token<'src>>>,
            ) -> $type {
                let nodes = $misc::$m_parse(iter);

                let $nodes = nodes;
                $(let $c_arg = $arg;)*
                $closure
            }
        }
    };
}

#[inline]
const fn token_field(t: &Token) -> bool {
    matches!(t, Token::Field { .. })
}

define_misc_parser! {
    name:      FieldParser,
    type:      FieldNode<'src>,
    fn:        parse_fields,
    token:     Token::Field,
    fields:    [ name, kind ],
    predicate: token_field,
    ctor:      |name, kind| {
        let kind = match kind {
            IdentifierOrArray::Identifier(i) => FieldTypeNode::Type(i),
            IdentifierOrArray::Array(i, s) => FieldTypeNode::Array(ArrayNode { kind: i, size: s }),
        };
        FieldNode { name, kind }
    }
}

#[inline]
const fn token_stream_field(t: &Token) -> bool {
    matches!(t, Token::StreamField { .. })
}

define_misc_parser! {
    name:      StreamFieldParser,
    type:      StreamFieldNode<'src>,
    fn:        parse_fields,
    token:     Token::StreamField,
    fields:    [ name, kind ],
    predicate: token_stream_field,
    ctor:      |name, kind| {
        StreamFieldNode { name, kind: StreamFieldTypeNode::new(kind) }
    }
}
