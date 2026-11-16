use miette::NamedSource;

use crate::error::ProtocolError;

#[derive(Default, Debug)]
pub struct Diagnostics {
    errors: Vec<ProtocolError>,
}

impl Diagnostics {
    pub fn report_err(&mut self, error: impl Into<ProtocolError>) {
        self.errors.push(error.into());
    }

    #[must_use]
    pub const fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print(&self, file: &str, content: &str) {
        let content = content.trim();
        for err in &self.errors {
            eprintln!(
                "{:?}\n",
                miette::Report::new(err.clone()).with_source_code(
                    NamedSource::new(file, content.to_string()).with_language("CMP")
                )
            );
        }
    }

    #[must_use]
    pub const fn errors(&self) -> &[ProtocolError] {
        self.errors.as_slice()
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<ProtocolError> {
        self.errors
    }
}

impl From<Vec<ProtocolError>> for Diagnostics {
    fn from(value: Vec<ProtocolError>) -> Self {
        Self { errors: value }
    }
}
