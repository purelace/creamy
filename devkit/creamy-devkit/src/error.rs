use creamy_utils::version;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Manifest file not found")]
    MissingManifest,

    #[error("{0} is not a file")]
    NotAFile(String),

    #[error("{0} is not a directory")]
    NotADirectory(String),

    #[error("{0}")]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    Manifest(#[from] creamy_manifest::ManifestError),

    #[error("{0}")]
    Version(#[from] version::VersionError),

    #[error("{0}")]
    BinRw(#[from] binrw::Error),
}
