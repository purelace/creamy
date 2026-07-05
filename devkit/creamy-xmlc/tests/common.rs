#![allow(clippy::missing_errors_doc)]

use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    ProtocolDefinition,
    error::{AstError, ProtocolError, SyntaxError},
};
use miette::SourceSpan;

#[must_use]
pub fn get_xml(version: &str, content: &str) -> String {
    format!(
        r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="test" version="{version}">
    {content}
</protocol>"#
    )
}

pub fn compile(content: &str) -> Result<ProtocolDefinition, Vec<ProtocolError>> {
    match creamy_xmlc::compile(&mut StringPool::default(), content) {
        Ok(value) => Ok(value),
        Err(err) => Err(err.into_inner()),
    }
}

#[must_use]
pub const fn zero_span() -> SourceSpan {
    unsafe { std::mem::zeroed() }
}

pub fn zeroize_span(err: &mut ProtocolError) {
    match err {
        ProtocolError::SyntaxError(syntax) => match syntax {
            SyntaxError::Xml { span, error: _ }
            | SyntaxError::UnknownTag { span }
            | SyntaxError::MissingAttribute {
                tag: _,
                attr: _,
                span,
            }
            | SyntaxError::InvalidVersionFormat { span }
            | SyntaxError::InvalidMajor { span }
            | SyntaxError::InvalidMinor { span }
            | SyntaxError::InvalidAccess { span }
            | SyntaxError::InvalidArraySyntax { span }
            | SyntaxError::IntParse { span, error: _ }
            | SyntaxError::EmptyIdentifier { span }
            | SyntaxError::InvalidIdentifier { span }
            | SyntaxError::NotANumber { span } => *span = zero_span(),
        },
        ProtocolError::AstError(ast) => {
            if let AstError::UnexpectedToken { span } = ast {
                *span = zero_span();
            }
        }
        ProtocolError::SemanticError(_) => {}
    }
}

pub fn zeroize_errors(errors: &mut Vec<ProtocolError>) {
    for err in errors {
        zeroize_span(err);
    }
}

#[macro_export]
macro_rules! assert_diag {
    ($left: expr, $right: expr, $content: expr $(,)?) => {
        let mut o = $left.clone();
        common::zeroize_errors(&mut o);
        pretty_assertions::assert_eq!(o, $right);
        creamy_xmlc::Diagnostics::from($left).print($content);
    };
}
