mod bitset;
mod enums;
mod flags;
mod message;
pub mod nodes;
mod structs;

use std::{iter::Peekable, vec::Drain};

use creamy_protocol_model::{Access, Version, error::Fallback};
pub use message::{StreamFieldNode, StreamFieldTypeNode};

use self::{
    flags::FlagsParser,
    message::{MessageParser, StreamParser},
    nodes::{
        ArrayNode, BitsetNode, EnumNode, FieldNode, FieldTypeNode, FlagsNode, GlobalTypesNode,
        GroupNode, MessageNodeType, StructNode,
    },
};
use crate::{
    Diagnostics,
    error::AstError,
    tokenizer::{Identifier, IdentifierOrArray, Token},
    tree::{bitset::BitsetParser, enums::EnumParser, structs::StructParser},
};

#[derive(Default)]
struct GroupBuilder<'src> {
    messages: Vec<MessageNodeType<'src>>,
    structs: Vec<StructNode<'src>>,
    enums: Vec<EnumNode<'src>>,
    flags: Vec<FlagsNode<'src>>,
    bitsets: Vec<BitsetNode<'src>>,
}

impl<'src> GroupBuilder<'src> {
    fn build_global(&mut self) -> GlobalTypesNode<'src> {
        GlobalTypesNode {
            structs: core::mem::take(&mut self.structs),
            enums: core::mem::take(&mut self.enums),
            flags: core::mem::take(&mut self.flags),
            bitsets: core::mem::take(&mut self.bitsets),
        }
    }

    fn build_local(&mut self, name: Identifier<'src>, access: Access) -> GroupNode<'src> {
        GroupNode {
            name,
            access,
            structs: core::mem::take(&mut self.structs),
            enums: core::mem::take(&mut self.enums),
            flags: core::mem::take(&mut self.flags),
            bitsets: core::mem::take(&mut self.bitsets),
            messages: core::mem::take(&mut self.messages),
        }
    }

    /// Пытается распарсить общий тип (struct, enum, flags, bitset).
    /// Возвращает `true`, если токен был распознан и обработан.
    fn try_parse_type(&mut self, iter: &mut Peekable<Drain<'_, Token<'src>>>) -> bool {
        match iter.peek() {
            Some(&Token::Struct { name, .. }) => {
                iter.next();
                self.structs.push(StructParser::parse_struct(name, iter));
            }
            Some(&Token::Enum { name, repr, .. }) => {
                iter.next();
                self.enums.push(EnumParser::parse_enum(name, repr, iter));
            }
            Some(&Token::Flags { name, .. }) => {
                iter.next();
                self.flags.push(FlagsParser::parse_flags(name, iter));
            }
            Some(&Token::Bitset { name, .. }) => {
                iter.next();
                self.bitsets.push(BitsetParser::parse_bitset(name, iter));
            }
            _ => return false,
        }
        true
    }

    fn try_parse_message(&mut self, iter: &mut Peekable<Drain<'_, Token<'src>>>) -> bool {
        match iter.peek() {
            Some(&Token::Message {
                name, direction, ..
            }) => {
                iter.next();
                self.messages
                    .push(MessageNodeType::Single(MessageParser::parse_single(
                        name, direction, iter,
                    )));
            }
            Some(&Token::Stream {
                name,
                timeout,
                direction,
                ..
            }) => {
                iter.next();
                self.messages
                    .push(MessageNodeType::Stream(StreamParser::parse_stream(
                        name, timeout, direction, iter,
                    )));
            }
            _ => return false,
        }

        true
    }
}

pub struct ProtocolTree<'src> {
    pub name: Identifier<'src>,
    pub version: Version,
    pub global: GlobalTypesNode<'src>,
    pub groups: Vec<GroupNode<'src>>,
}

impl<'src> ProtocolTree<'src> {
    pub fn new(mut tokens: Vec<Token<'src>>, diagnostics: &mut Diagnostics) -> Self {
        let mut iter = tokens.drain(..).peekable();

        let (name, version) = if let Some(Token::Protocol { name, version, .. }) = iter.next() {
            (name, version)
        } else {
            diagnostics.report_err(AstError::MissingProtocolToken);
            (Identifier::fallback(), Version::fallback())
        };

        let mut global = GlobalTypesNode::default();
        let mut builder = GroupBuilder::default();
        let mut groups = Vec::<GroupNode>::new();

        while let Some(token) = iter.next() {
            match token {
                Token::Global { .. } => {
                    while builder.try_parse_type(&mut iter) {}
                    global = builder.build_global();
                }
                Token::Group { name, access, .. } => {
                    loop {
                        if builder.try_parse_type(&mut iter) {
                            continue;
                        }

                        if builder.try_parse_message(&mut iter) {
                            continue;
                        }

                        break;
                    }
                    groups.push(builder.build_local(name, access));
                }
                other => {
                    diagnostics.report_err(AstError::UnexpectedToken { span: other.span() });
                }
            }
        }

        ProtocolTree {
            name,
            version,
            global,
            groups,
        }
    }

    /// Значение не может превышать [``crate::constraints::MAX_TYPE_COUNT``]
    #[allow(clippy::cast_possible_truncation)]
    pub fn type_count(&self) -> u16 {
        self.global.type_count() + self.groups.iter().map(GroupNode::type_count).sum::<u16>()
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
