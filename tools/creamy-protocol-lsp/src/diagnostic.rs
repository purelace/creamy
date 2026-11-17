use creamy_xmlc::{
    Diagnostics,
    error::{ProtocolError, SyntaxError},
};
use tower_lsp::lsp_types::{Diagnostic, Range};

/*
fn syntax_to_diag(e: SyntaxError) -> Diagnostic {
    match e {
        SyntaxError::InvalidVersionFormat { span } => todo!(),
        SyntaxError::InvalidAccess { span } => todo!(),
        SyntaxError::InvalidDirection { span } => todo!(),
        SyntaxError::IntParse { span, error } => todo!(),
        SyntaxError::NotANumber { span } => todo!(),
        SyntaxError::UnexpectedToken { expected, span } => {
            Diagnostic::new(Range::new(span.offset(), span.len().into()), )
        },
        SyntaxError::UnexpectedEOF => todo!(),
        SyntaxError::UnknownKeyword { actual, span } => todo!(),
    }
}

fn to_tower_diag(d: Diagnostics) -> Vec<Diagnostic> {
    let mut vec = Vec::with_capacity(d.errors().len());

    for e in d.errors() {
        match e {
            ProtocolError::SyntaxError(e) => match e {
                SyntaxError::InvalidVersionFormat { span } => Diagnostic::new(
                    range: Range,
                    severity: Option<DiagnosticSeverity>,
                    code: Option<NumberOrString>,
                    source: Option<String>,
                    message: String,
                    related_information: Option<Vec<DiagnosticRelatedInformation>>,
                    tags: Option<Vec<DiagnosticTag>>,
                ),
                SyntaxError::InvalidAccess { span } => todo!(),
                SyntaxError::InvalidDirection { span } => todo!(),
                SyntaxError::IntParse { span, error } => todo!(),
                SyntaxError::NotANumber { span } => todo!(),
                SyntaxError::UnexpectedToken { expected, span } => todo!(),
                SyntaxError::UnexpectedEOF => todo!(),
                SyntaxError::UnknownKeyword { actual, span } => todo!(),
            },
            ProtocolError::SemanticError(e) => todo!(),
        }
    }
}
*/
