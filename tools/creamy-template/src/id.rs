use std::fmt::Display;

use crate::error::TemplateError;

/// A wrapper around a validated identifier string.
///
/// The identifier must follow the format `segment1.segment2.segment3`,
/// where each segment starts with an alphabetic character or an underscore
/// and contains only alphanumeric characters or underscores.
pub struct Id(String);
impl Id {
    /// Creates a new `Id` by validating the provided string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string does not conform to the required
    /// three-segment identifier format.
    pub fn new(value: impl Into<String>) -> Result<Self, TemplateError> {
        let value = value.into();
        if value.is_empty() {
            return Err(TemplateError::EmptyInput);
        }

        let regex = lazy_regex::regex!(
            r"^[_a-zA-Z][_a-zA-Z0-9]*\.[_a-zA-Z][_a-zA-Z0-9]*\.[_a-zA-Z][_a-zA-Z0-9]*$"
        );
        if regex.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(TemplateError::InvalidIdFormat)
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
