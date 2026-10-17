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
    Binary(Box<dyn core::error::Error + Send + Sync + 'static>),

    #[error("Too many files")]
    TooManyFiles,
}
