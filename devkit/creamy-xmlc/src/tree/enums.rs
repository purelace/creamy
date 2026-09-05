use super::nodes::{EnumNode, VariantNode};
use crate::{
    StringPoolIntern, define_misc_parser, define_toplevel_parser,
    error::AstError,
    tokenizer::{Identifier, Token},
    utils::VariantsRange,
};

#[inline]
const fn token_variant(t: &Token) -> bool {
    matches!(t, Token::Variant { .. })
}

define_misc_parser! {
    name:      VariantParser,
    type:      VariantNode,
    return:    VariantsRange,
    error:     AstError::TooManyVariants,
    fn:        parse_variants,
    builder:   build_variants,
    token:     Token::Variant,
    fields:    [ name, value, span ],
    predicate: token_variant,
    ctor:      |pool, name, kind, span| {
        VariantNode::new(name.intern(pool), value)
    }
}

define_toplevel_parser!(
    name:       EnumParser,
    type:       EnumNode,
    error:      AstError::TooManyEnums,
    top_parse:  parse_enum,
    misc_parse: variant.parse_variants,
    args:       [ name: Identifier, repr: Identifier ],
    ctor:       |ctx, range, name, repr| EnumNode::new(
        name.intern(ctx.pool),
        repr.intern(ctx.pool),
        range
    )
);
