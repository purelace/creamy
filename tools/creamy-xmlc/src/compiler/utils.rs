use super::Context;
use crate::{ast::AttributeParameterValueNode, error::SyntaxError, tokenizer::Span};

pub fn consume_string<'src>(
    context: &mut Context,
    value: &AttributeParameterValueNode<'src>,
) -> Option<(&'src str, Span)> {
    match value {
        AttributeParameterValueNode::Number(n) => {
            context.diag.report_err::<()>(SyntaxError::UnexpectedToken {
                expected: "string",
                span: n.span.into(),
            });
            None
        }
        AttributeParameterValueNode::String(n) => Some((n.value, n.span)),
    }
}
