//#![deny(clippy::unwrap_used)]

pub mod config;
mod inner;
mod progress;
mod utils;
mod watcher;

use creamy_engine_core::{
    PluginLoader,
    devkit::{self, BinaryPlugin},
};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use self::{config::ValidLoaderConfig, inner::InnerLoader};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Devkit(#[from] devkit::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

fn load_package(data: &[u8]) -> Result<BinaryPlugin> {
    let package = BinaryPlugin::load_from_bytes(data)?;
    tracing::info!(
        "[Loader] Plugin '{name}@{version}' loaded",
        name = package.manifest().name(),
        version = package.version()
    );
    Ok(package)
}

pub struct AsyncLoader {
    receiver: UnboundedReceiver<BinaryPlugin>,
    _inner: JoinHandle<()>,
}

impl PluginLoader for AsyncLoader {
    fn preload(&mut self) {}

    #[allow(clippy::cast_possible_truncation)]
    fn loaded(&self) -> u32 {
        self.receiver.len() as u32
    }

    fn take_loaded_package(&mut self) -> Option<BinaryPlugin> {
        self.receiver.try_recv().ok()
    }
}

impl AsyncLoader {
    pub async fn new(
        config: ValidLoaderConfig,
        async_runtime: tokio::runtime::Handle,
    ) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut inner = InnerLoader::new(config).await?;
        let inner = async_runtime.spawn(async move {
            loop {
                inner.poll_and_load().await;

                let loaded_plugins = inner.take_loaded();
                if !loaded_plugins.is_empty() {
                    for plugin in loaded_plugins {
                        let _ = tx.send(plugin);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            receiver: rx,
            _inner: inner,
        })
    }
}
