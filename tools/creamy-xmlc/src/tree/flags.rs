use super::nodes::{FlagsNode, OptionNode};
use crate::{
    define_misc_parser, define_toplevel_parser,
    tokenizer::{Identifier, Token},
};

#[inline]
const fn token_option(t: &Token) -> bool {
    matches!(t, Token::Option { .. })
}

define_misc_parser! {
    name:      OptionParser,
    type:      OptionNode<'src>,
    fn:        parse_options,
    token:     Token::Option,
    fields:    [ name ],
    predicate: token_option,
    ctor:      |name| {
        OptionNode { name }
    }
}

define_toplevel_parser!(
    name:       FlagsParser,
    type:       FlagsNode<'src>,
    top_parse:  parse_flags,
    misc_parse: OptionParser::parse_options,
    args:       [ name: Identifier<'src> ],
    ctor:       |options, name|  {
        FlagsNode {
            name,
            options,
        }
    }
);
