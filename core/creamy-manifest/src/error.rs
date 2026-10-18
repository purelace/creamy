use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("Field '{0}' cannot be empty")]
    EmptyValue(&'static str),

    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}
