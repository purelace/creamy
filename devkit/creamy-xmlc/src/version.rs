use std::fmt::Display;

use binrw::{BinRead, BinWrite};
use miette::SourceSpan;

use crate::error::{Fallback, SyntaxError};

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Version {
    pub fn new(s: &str, at_f: impl Fn() -> SourceSpan) -> Result<Self, SyntaxError> {
        let mut parts = s.split('.');
        let (major, minor) = match (parts.next(), parts.next(), parts.next()) {
            (Some(major), Some(minor), None) => (major.trim(), minor.trim()),
            _ => return Err(SyntaxError::InvalidVersionFormat { span: at_f() }),
        };

        Ok(Version {
            major: major
                .parse()
                .map_err(|_| SyntaxError::InvalidMajor { span: at_f() })?,
            minor: minor
                .parse()
                .map_err(|_| SyntaxError::InvalidMinor { span: at_f() })?,
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Fallback for Version {
    fn fallback() -> Self {
        Self::default()
    }
}
