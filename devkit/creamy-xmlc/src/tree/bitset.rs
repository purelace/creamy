use super::nodes::{BitsetNode, BitsetValueNode};
use crate::{
    StringPoolIntern, define_misc_parser, define_toplevel_parser,
    error::AstError,
    tokenizer::{Identifier, Token},
    utils::BitsetValuesRange,
};

#[inline]
const fn token_bvalue(t: &Token) -> bool {
    matches!(t, Token::BValue { .. })
}

define_misc_parser! {
    name:      BValueParser,
    type:      BValueNode,
    return:    BitsetValuesRange,
    error:     AstError::TooManyBitsetValues,
    fn:        parse_bvalues,
    builder:   build_bvalues,
    token:     Token::BValue,
    fields:    [ name, bits, repr, span ],
    predicate: token_bvalue,
    ctor:      |pool, name, bits, repr, span| {
        BitsetValueNode::new(name.intern(pool), repr.intern(pool), bits)
    }
}

define_toplevel_parser!(
    name:       BitsetParser,
    type:       BitsetNode,
    error:      AstError::TooManyBitsets,
    top_parse:  parse_bitset,
    misc_parse: bvalue.parse_bvalues,
    args:       [ name: Identifier ],
    ctor:       |ctx, range, name| BitsetNode::new(
        name.intern(ctx.pool),
        range
    )
);
