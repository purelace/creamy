use std::num::ParseIntError;

use creamy_protocol_model::error::{Fallback, SemanticError};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::Diagnostics;

#[derive(Debug, Error, Diagnostic, Clone, PartialEq, Eq)]
#[diagnostic(severity(Error))]
pub enum SyntaxError {
    #[error("[P002] Missing '{attr}' attribute in <{tag}> tag")]
    #[diagnostic(code(syntax::missing_attribute))]
    #[diagnostic(help("add {attr}=\"...\" attribute"))]
    MissingAttribute {
        tag: &'static str,
        attr: &'static str,
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P003] Invalid version format")]
    #[diagnostic(code(syntax::invalid_version_format))]
    #[diagnostic(help("allowed syntax: MAJOR.MINOR.PATCH"))]
    InvalidVersionFormat {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P006] Invalid access value")]
    #[diagnostic(code(syntax::invalid_access))]
    #[diagnostic(help("allowed values: Public, Private"))]
    InvalidAccess {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P007] Invalid direction value")]
    #[diagnostic(code(syntax::invalid_direction))]
    #[diagnostic(help("allowed values: Incoming, Outgoing, Duplex"))]
    InvalidDirection {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P008] Invalid syntax")]
    #[diagnostic(code(syntax::invalid_array_syntax))]
    #[diagnostic(help("should be [TYPE; SIZE]"))]
    InvalidArraySyntax {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P009] Int parse error: {error}")]
    #[diagnostic(code(syntax::int_error))]
    IntParse {
        #[label("Here")]
        span: SourceSpan,
        error: ParseIntError,
    },

    #[error("[P010] Empty identifier")]
    #[diagnostic(code(syntax::empty_identifier))]
    #[diagnostic(help("Identifier cannot be empty"))]
    EmptyIdentifier {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P011] Invalid identifier")]
    #[diagnostic(code(syntax::invalid_identifier))]
    InvalidIdentifier {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("[P012] Not a number")]
    #[diagnostic(code(syntax::nan))]
    NotANumber {
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("Syntax Error: expected '{expected}'")]
    UnexpectedToken {
        expected: &'static str,
        #[label("Here")]
        span: SourceSpan,
    },

    #[error("Syntax Error: expected an item")]
    UnexpectedEOF,

    #[error("Syntax Error: unknown keyword '{actual}'")]
    UnknownKeyword {
        actual: String,

        #[label("Here")]
        span: SourceSpan,
    },
}

#[derive(Debug, Error, Diagnostic, Clone, PartialEq, Eq)]
#[diagnostic(severity(Error))]
pub enum AstError {
    #[error("[P100] Missing protocol token")]
    #[diagnostic(code(ast::missing_protocol_token))]
    #[diagnostic(help("Expected <protocol> token"))]
    MissingProtocolToken,

    #[error("[P101] Unexpected token")]
    #[diagnostic(code(ast::unexpected_token))]
    //#[diagnostic(help("expected {should_be}"))]
    UnexpectedToken {
        #[label("Here")]
        span: SourceSpan,
        //should_be: String,
    },
}

#[derive(Debug, Error, Diagnostic, Clone, PartialEq, Eq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum ProtocolError {
    SyntaxError(#[from] SyntaxError),
    AstError(#[from] AstError),
    SemanticError(#[from] SemanticError),
}

pub trait ProtocolErrorExt<T: Fallback> {
    fn or_recover(self, diagnostics: &mut Diagnostics) -> T;
}

impl<T: Fallback, E: Into<ProtocolError>> ProtocolErrorExt<T> for Result<T, E> {
    fn or_recover(self, diagnostics: &mut Diagnostics) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                diagnostics.report_err(error);
                T::fallback()
            }
        }
    }
}
