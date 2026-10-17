#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`.creamy` directory not found")]
    MissingDirectory,

    #[error("{0}")]
    Manifest(#[from] creamy_manifest::ManifestError),

    #[error("{0}")]
    Binary(alloc::boxed::Box<dyn core::error::Error + Send + Sync + 'static>),

    #[cfg(feature = "compile")]
    #[error("Too many files")]
    TooManyFiles,

    #[cfg(feature = "compile")]
    #[error("{0}")]
    IO(#[from] std::io::Error),
}
