use std::str::FromStr;

use creamy_protocol_model::Version;
use miette::SourceSpan;

use crate::error::SyntaxError;

pub fn parse_version(s: &str, at_f: impl Fn() -> SourceSpan) -> Result<Version, SyntaxError> {
    Version::from_str(s).map_err(|_| SyntaxError::InvalidVersionFormat { span: at_f() })
}
