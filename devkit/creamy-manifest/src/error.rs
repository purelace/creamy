use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Field '{0}' cannot be empty")]
    EmptyValue(&'static str),

    #[error("{0}")]
    Version(#[from] semver::Error),

    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

impl Eq for ManifestError {}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PartialEq for ManifestError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EmptyValue(l0), Self::EmptyValue(r0)) => l0 == r0,
            (Self::Version(l0), Self::Version(r0)) => l0.to_string() == r0.to_string(),
            (Self::Toml(l0), Self::Toml(r0)) => l0 == r0,
            _ => false,
        }
    }
}
