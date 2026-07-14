use thiserror::Error;

pub type Result<T> = core::result::Result<T, EngineError>;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("{0}")]
    Wasmtime(wasmtime::Error),
}
