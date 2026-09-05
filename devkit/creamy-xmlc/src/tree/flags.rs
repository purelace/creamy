use super::nodes::{FlagsNode, OptionNode};
use crate::{
    StringPoolIntern, define_misc_parser, define_toplevel_parser,
    error::AstError,
    tokenizer::{Identifier, Token},
    utils::OptionsRange,
};

#[inline]
const fn token_option(t: &Token) -> bool {
    matches!(t, Token::Option { .. })
}

define_misc_parser! {
    name:      OptionParser,
    type:      OptionNode,
    return:    OptionsRange,
    error:     AstError::TooManyOptions,
    fn:        parse_options,
    builder:   build_options,
    token:     Token::Option,
    fields:    [ name, span ],
    predicate: token_option,
    ctor:      |pool, name, span| {
        OptionNode::new(name.intern(pool))
    }
}

define_toplevel_parser!(
    name:       FlagsParser,
    type:       FlagsNode,
    error:      AstError::TooManyFlags,
    top_parse:  parse_flags,
    misc_parse: option.parse_options,
    args:       [ name: Identifier ],
    ctor:       |ctx, range, name| FlagsNode::new(
        name.intern(ctx.pool),
        range
    )
);
