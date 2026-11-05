#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Devkit(#[from] creamy_engine_core::package::Error),

    #[error("{0}")]
    Package(#[from] creamy_binary::ReadError<!>),
}
