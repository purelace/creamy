mod watcher;

use std::path::{Path, PathBuf};

use creamy_engine_core::{PluginLoader, devkit::BinaryPlugin};

use self::watcher::FileWatcher;

fn load_file(file: PathBuf) -> Result<BinaryPlugin, std::io::Error> {
    tracing::info!("[Loader] Loading package: {}", file.display());
    let bytes = std::fs::read(file)?;

    let package = BinaryPlugin::load_from_bytes(&bytes).unwrap();
    tracing::info!(
        "[Loader] Plugin '{name}@{version}' loaded",
        name = package.manifest().name(),
        version = package.version()
    );
    Ok(package)
}

pub struct Loader {
    watcher: FileWatcher,
    loaded: Vec<BinaryPlugin>,
}

impl Loader {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Ok(Self {
            watcher: FileWatcher::new(directory.as_ref())?,
            loaded: vec![],
        })
    }
}

impl PluginLoader for Loader {
    fn preload(&mut self) {}

    fn load(&mut self) {
        let mut loaded = std::mem::take(&mut self.loaded);
        match self.watcher.try_read_events() {
            Ok(iter) => {
                for path in iter {
                    match load_file(path) {
                        Ok(package) => loaded.push(package),
                        Err(e) => {
                            tracing::error!("y:{e}");
                        }
                    }
                }
            }
            Err(e) => tracing::error!("x: {e}"),
        }

        self.loaded = loaded;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn loaded(&self) -> u32 {
        self.loaded.len() as u32
    }

    fn take_loaded_package(&mut self) -> Option<BinaryPlugin> {
        self.loaded.pop()
    }
}
