mod watcher;

use std::{io::ErrorKind, path::PathBuf};

use creamy_binary::BinaryReader;
use creamy_engine_core::package::BinaryPlugin;

use self::watcher::FileWatcher;
use super::{PluginLoader, config::ValidLoaderConfig};

fn load_file(path: PathBuf) -> Result<BinaryPlugin, std::io::Error> {
    log::info!("Loading package: {}", path.display());
    let file = std::fs::File::open(path)?;
    let mut file = creamy_binary::adapters::FromStd::new(file);

    let package = BinaryPlugin::read_ne(&mut file).unwrap();
    log::info!(
        "Plugin '{name}@{version}' loaded",
        name = package.manifest.name(),
        version = package.version
    );
    Ok(package)
}

pub struct Loader {
    watcher: FileWatcher,
    loaded: Vec<BinaryPlugin>,
}

impl Loader {
    /// Creates a new [`Loader`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying inotify instance fails to initialize.
    pub fn new(config: ValidLoaderConfig) -> Result<Self, std::io::Error> {
        Ok(Self {
            watcher: FileWatcher::new(config.plugin_directory)?,
            loaded: vec![],
        })
    }
}

impl PluginLoader for Loader {
    fn tick(&mut self) {
        let mut loaded = std::mem::take(&mut self.loaded);
        match self.watcher.try_read_events() {
            Ok(iter) => {
                for path in iter {
                    match load_file(path) {
                        Ok(package) => loaded.push(package),
                        Err(e) => {
                            log::error!("y:{e}");
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    log::error!("x: {e}");
                }
            }
        }

        self.loaded = loaded;
    }

    fn take_loaded_package(&mut self) -> Option<BinaryPlugin> {
        self.loaded.pop()
    }
}
