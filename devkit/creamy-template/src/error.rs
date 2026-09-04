use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    /// Error occurred during I/O operations (e.g., creating directories or writing files).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The provided identifier does not follow the required format.
    #[error("Invalid identifier format")]
    InvalidIdFormat,

    /// The input string is empty.
    #[error("Input string cannot be empty")]
    EmptyInput,
}
