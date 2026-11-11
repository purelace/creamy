use super::nodes::{EnumNode, VariantNode};
use crate::{
    define_misc_parser, define_toplevel_parser,
    tokenizer::{Identifier, Token},
};

#[inline]
const fn token_variant(t: &Token) -> bool {
    matches!(t, Token::Variant { .. })
}

define_misc_parser! {
    name:      VariantParser,
    type:      VariantNode<'src>,
    fn:        parse_variants,
    token:     Token::Variant,
    fields:    [ name, value ],
    predicate: token_variant,
    ctor:      |name, value | {
        VariantNode { name, value }
    }
}

define_toplevel_parser!(
    name:       EnumParser,
    type:       EnumNode<'src>,
    top_parse:  parse_enum,
    misc_parse: VariantParser::parse_variants,
    args:       [ name: Identifier<'src>, repr: Identifier<'src> ],
    ctor:       |variants, name, repr| {
        EnumNode { name, repr, variants }
    }
);
