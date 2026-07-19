use std::str::FromStr;

use miette::SourceSpan;
use semver::Version;

use crate::error::{Fallback, SyntaxError};

pub fn parse_version(s: &str, at_f: impl Fn() -> SourceSpan) -> Result<Version, SyntaxError> {
    Version::from_str(s).map_err(|_| SyntaxError::InvalidVersionFormat { span: at_f() })
}

impl Fallback for Version {
    fn fallback() -> Self {
        Version::new(0, 0, 0)
    }
}
