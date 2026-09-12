use thiserror::Error;

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

    #[error("{0}")]
    BinRw(#[from] binrw::Error),

    #[error("Too many files")]
    TooManyFiles,
}
