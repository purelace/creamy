use super::{FieldParser, nodes::StructNode};
use crate::{define_toplevel_parser, tokenizer::Identifier};

define_toplevel_parser!(
    name:       StructParser,
    type:       StructNode<'src>,
    top_parse:  parse_struct,
    misc_parse: FieldParser::parse_fields,
    args:       [ name: Identifier<'src> ],
    ctor:       |fields, name|  {
        StructNode { name, fields }
    }
);
