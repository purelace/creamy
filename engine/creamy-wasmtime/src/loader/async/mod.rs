mod error;
mod inner;
mod progress;
mod watcher;

use creamy_binary::BinaryReader;
use creamy_engine_core::package::BinaryPlugin;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use self::{
    error::Error,
    inner::{InnerLoader, LoadingContext},
};
use super::{PluginLoader, config::ValidLoaderConfig};

pub type Loader = AsyncLoader;

fn load_package(mut data: &[u8]) -> Result<BinaryPlugin, Error> {
    let package = BinaryPlugin::read_ne(&mut data)?;
    tracing::info!(
        "Plugin '{name}@{version}' loaded",
        name = package.manifest.name(),
        version = package.version
    );
    Ok(package)
}

pub struct AsyncLoader {
    loaded: UnboundedReceiver<BinaryPlugin>,
    #[cfg(feature = "ui-events")]
    in_progress: UnboundedReceiver<LoadingContext>,
    _inner: JoinHandle<()>,
}

impl PluginLoader for AsyncLoader {
    fn tick(&mut self) {}

    fn take_loaded_package(&mut self) -> Option<BinaryPlugin> {
        self.loaded.try_recv().ok()
    }
}

impl AsyncLoader {
    pub async fn new(
        config: ValidLoaderConfig,
        async_runtime: tokio::runtime::Handle,
    ) -> Result<Self, Error> {
        let (in_progress_tx, in_progress_rx) = tokio::sync::mpsc::unbounded_channel();

        let (loaded_tx, loaded_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut inner = InnerLoader::new(config, in_progress_tx).await?;
        let inner = async_runtime.spawn(async move {
            loop {
                inner.poll_and_load().await;

                let loaded_plugins = inner.take_loaded();
                if !loaded_plugins.is_empty() {
                    for plugin in loaded_plugins {
                        let _ = loaded_tx.send(plugin);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Self {
            loaded: loaded_rx,
            in_progress: in_progress_rx,
            _inner: inner,
        })
    }

    #[cfg(feature = "ui-events")]
    pub fn get_in_progress(&mut self, buffer: &mut Vec<LoadingContext>) -> usize {
        let mut read = 0;
        while let Ok(c) = self.in_progress.try_recv() {
            buffer.push(c);
            read += 1;
        }

        read
    }
}
