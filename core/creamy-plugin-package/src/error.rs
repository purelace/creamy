#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`.creamy` directory not found")]
    MissingDirectory,

    #[error("{0}")]
    Manifest(#[from] creamy_manifest::ManifestError),

    #[error("{0}")]
    Binary(alloc::boxed::Box<dyn core::error::Error + Send + Sync + 'static>),
}
