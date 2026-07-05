use crate::{
    StringPoolIntern, define_toplevel_parser, error::AstError, nodes::StructNode,
    tokenizer::Identifier,
};

define_toplevel_parser!(
    name:       StructParser,
    type:       StructNode,
    error:      AstError::TooManyStructs,
    top_parse:  parse_struct,
    misc_parse: field.parse_fields,
    args:       [ name: Identifier ],
    ctor:       |ctx, range, name| StructNode::new(name.intern(ctx.pool), range)
);
