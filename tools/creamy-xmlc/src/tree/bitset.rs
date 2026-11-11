use crate::{
    define_misc_parser, define_toplevel_parser,
    tokenizer::{Identifier, Token},
    tree::nodes::{BitsetNode, BitsetValueNode},
};

#[inline]
const fn token_bvalue(t: &Token) -> bool {
    matches!(t, Token::BValue { .. })
}

define_misc_parser! {
    name:      BValueParser,
    type:      BitsetValueNode<'src>,
    fn:        parse_bvalues,
    token:     Token::BValue,
    fields:    [ name, bits, repr],
    predicate: token_bvalue,
    ctor:      |name, bits, repr| {
        BitsetValueNode {
            name,
            repr,
            bits,
        }
    }
}

define_toplevel_parser!(
    name:       BitsetParser,
    type:       BitsetNode<'src>,
    top_parse:  parse_bitset,
    misc_parse: BValueParser::parse_bvalues,
    args:       [ name: Identifier<'src> ],
    ctor:       |values, name|
    BitsetNode {
        name,
        values
    }
);
