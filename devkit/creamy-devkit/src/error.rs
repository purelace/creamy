use thiserror::Error;

pub struct BinRwError(binrw::Error);

impl core::error::Error for BinRwError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl core::fmt::Debug for BinRwError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::fmt::Display for BinRwError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <binrw::Error as core::fmt::Display>::fmt(&self.0, f)
    }
}

impl From<binrw::Error> for BinRwError {
    fn from(value: binrw::Error) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("`.creamy` directory not found")]
    MissingDirectory,

    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    Manifest(#[from] creamy_manifest::ManifestError),

    #[error("{0}")]
    Version(#[from] semver::Error),

    #[error(transparent)]
    BinRw(#[from] BinRwError),
    #[error("Too many files")]
    TooManyFiles,
}
