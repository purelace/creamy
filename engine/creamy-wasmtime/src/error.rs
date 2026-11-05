#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("{0}")]
    Wasmtime(#[from] wasmtime::Error),
}
