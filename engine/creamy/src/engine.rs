use std::time::Duration;

use creamy_devkit::BinaryPlugin;
use garde::Validate;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{config::EngineConfig, loader::PluginLoader, runtime::Runtime};

pub struct PluginEngine {
    receiver: UnboundedReceiver<BinaryPlugin>,
    runtime: Runtime<()>,
}

#[allow(dead_code, unused)]
impl PluginEngine {
    pub async fn new(config: &EngineConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config.validate()?;

        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut loader = PluginLoader::new(&config.general).await?;

        tokio::spawn(async move {
            loop {
                loader.poll_and_load().await;

                let loaded_plugins = loader.take_loaded();
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
            runtime: Runtime::new(config.performance.heap_size)?,
        })
    }

    pub fn run(&mut self) {
        match self.receiver.try_recv() {
            Ok(plugin) => {
                self.runtime.init_module(plugin.core()).unwrap();
            }
            Err(err) => {}
        }

        self.runtime.tick();
        self.runtime.tick();

        // For testing only
        std::thread::sleep(Duration::from_millis(16));
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.runtime.loaded_plugins()
    }
}
