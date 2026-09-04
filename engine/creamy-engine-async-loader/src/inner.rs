use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    sync::{Arc, atomic::AtomicUsize},
};

use creamy_engine_core::{PACKAGE_FILE_EXTENSION, devkit::BinaryPlugin};
use inotify::{Event, EventMask};
use tokio::{
    io::AsyncReadExt,
    sync::{RwLock, Semaphore},
    task::JoinHandle,
};

use crate::{
    Result, config::ValidLoaderConfig, load_package, progress::ProgressReader, watcher::FileWatcher,
};

pub struct LoadingInProgress {
    progress: Arc<AtomicUsize>,
    data: Vec<u8>,
    is_done: bool,
}

pub struct InnerLoader {
    watcher: FileWatcher,
    directory: PathBuf,
    semaphore: Arc<Semaphore>,
    loading: Vec<JoinHandle<Option<BinaryPlugin>>>,
    loaded: Vec<BinaryPlugin>,
}

impl InnerLoader {
    pub async fn new(config: ValidLoaderConfig) -> Result<Self> {
        let mut instance = Self {
            semaphore: Arc::new(Semaphore::new(config.parallel_downloads as usize)),
            watcher: FileWatcher::new(&config.plugin_directory.to_string_lossy())?,
            directory: config.plugin_directory,
            loading: vec![],
            loaded: vec![],
        };
        instance.load_all_plugins_from_directory().await?;
        Ok(instance)
    }

    async fn load_all_plugins_from_directory(&mut self) -> Result<()> {
        let mut iter = tokio::fs::read_dir(&self.directory).await?;
        while let Some(entry) = iter.next_entry().await? {
            if entry.path().extension() == Some(OsStr::new(PACKAGE_FILE_EXTENSION)) {
                self.load_file(entry.path()).await;
            }
        }
        Ok(())
    }

    pub fn take_loaded(&mut self) -> Vec<BinaryPlugin> {
        std::mem::take(&mut self.loaded)
    }

    async fn store_loaded(
        loading: &mut Vec<JoinHandle<Option<BinaryPlugin>>>,
        loaded: &mut Vec<BinaryPlugin>,
    ) {
        let mut i = 0;
        while i < loading.len() {
            let handle = &mut loading[i];

            if handle.is_finished() {
                match handle.await {
                    Ok(package) => {
                        if let Some(package) = package {
                            loaded.push(package);
                        }
                    }
                    Err(error) => {
                        tracing::error!("{error}");
                    }
                }
                loading.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    async fn load_file(&mut self, file: PathBuf) {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();

        tracing::info!("[Loader] Loading package: {}", file.display());
        //let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
        let mut reader = ProgressReader::from_file(file).await;

        let state = Arc::new(RwLock::new(LoadingInProgress {
            progress: reader.counter(),
            data: Vec::with_capacity(reader.total()),
            is_done: false,
        }));

        // Mostly used for loading visualization
        // TODO: send event instead
        //assert!(self.in_progress.insert(file_name, state.clone()).is_none());

        let handle = tokio::spawn(async move {
            let mut state_write = state.write().await;
            reader.read_to_end(&mut state_write.data).await.unwrap();
            state_write.is_done = true;

            drop(permit);

            match load_package(&state_write.data) {
                Ok(package) => Some(package),
                Err(err) => {
                    tracing::error!("Got an error while the plugin is loading:");
                    println!("{err}");
                    None
                }
            }
        });

        self.loading.push(handle);
    }

    async fn validate_event(&self, event: Event<OsString>) -> Option<PathBuf> {
        match event.mask {
            //EventMask::MOVED_TO
            EventMask::DELETE | EventMask::MOVED_FROM => return None,
            _ => {}
        }

        let name = event.name?;
        let path = self.directory.join(name);
        let extension = path.extension()?;

        let is_valid_ext = extension == PACKAGE_FILE_EXTENSION;

        let is_valid = is_valid_ext
            && match tokio::fs::metadata(&path).await {
                Ok(metadata) => metadata.is_file(),
                Err(error) => {
                    eprintln!("Loader error: {error}");
                    false
                }
            };

        if !is_valid {
            return None;
        }

        Some(path)
    }

    pub async fn poll_and_load(&mut self) {
        let InnerLoader {
            watcher,
            directory: _,
            semaphore: _,
            loading,
            loaded,
        } = self;

        tokio::select! {
            _ = Self::store_loaded(loading, loaded) => {}

            maybe_event = watcher.run_file_watcher() => {
                if let Some(event) = maybe_event
                    && let Some(path) = self.validate_event(event).await {
                        self.load_file(path).await;
                    }
            }
        }
    }
}
