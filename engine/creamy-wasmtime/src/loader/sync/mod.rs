mod watcher;

use std::{
    io::{ErrorKind, Read},
    path::PathBuf,
};

use self::watcher::FileWatcher;
use super::{PluginLoader, config::ValidLoaderConfig};

fn load_file(path: PathBuf) -> Result<Box<[u8]>, std::io::Error> {
    log::info!("Loading package: {}", path.display());
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer.into_boxed_slice())

    //let package = BinaryPlugin::read_ne(&mut file).unwrap();
    //log::info!(
    //    "Plugin '{name}@{version}' loaded",
    //    name = package.manifest.name(),
    //    version = package.version
    //);
    //Ok(package)
}

pub struct Loader {
    watcher: FileWatcher,
    loaded: Vec<Box<[u8]>>,
}

impl Loader {
    /// Creates a new [`Loader`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying inotify instance fails to initialize.
    pub fn new(config: ValidLoaderConfig) -> Result<Self, std::io::Error> {
        let (watcher, found) = FileWatcher::new(config.plugin_directory)?;
        let mut loaded = Vec::new();
        for path in found {
            match load_file(path) {
                Ok(package) => loaded.push(package),
                Err(e) => {
                    log::error!("y:{e}");
                }
            }
        }
        Ok(Self { watcher, loaded })
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

    fn take_loaded_package(&mut self) -> Option<Box<[u8]>> {
        self.loaded.pop()
    }
}
