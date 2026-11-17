#![allow(clippy::missing_errors_doc)]

use creamy_xmlc::{
    SemanticModel,
    error::{ProtocolError, SemanticError, SyntaxError},
};
use miette::SourceSpan;

pub fn compile(content: &str) -> Result<SemanticModel, Vec<ProtocolError>> {
    match creamy_xmlc::SemanticModel::compile(content) {
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
            SyntaxError::InvalidVersionFormat { span }
            | SyntaxError::InvalidAccess { span }
            | SyntaxError::InvalidDirection { span }
            | SyntaxError::IntParse { span, .. }
            | SyntaxError::UnknownKeyword { span, .. }
            | SyntaxError::UnexpectedToken { span, .. }
            | SyntaxError::NotANumber { span } => *span = zero_span(),
            SyntaxError::UnexpectedEOF => (),
        },
        ProtocolError::SemanticError(e) => match e {
            SemanticError::ForbiddenType { span }
            | SemanticError::ZeroBitsetValueWidth { span }
            | SemanticError::BitsetValueExceedsRepr { span, .. }
            | SemanticError::NameDefinedMultipleTimes { span, .. } => *span = zero_span(),
            _ => {}
        },
    }
}

pub fn zeroize_errors(errors: &mut Vec<ProtocolError>) {
    for err in errors {
        zeroize_span(err);
    }
}

#[macro_export]
macro_rules! assert_diag {
    (
        $path: expr,
        $left: expr,
        $right: expr,
        $content: expr $(,)?) => {
        let mut o = $left.clone();
        common::zeroize_errors(&mut o);
        pretty_assertions::assert_eq!(o, $right);
        creamy_xmlc::Diagnostics::from($left).print($path, $content);
    };
}
