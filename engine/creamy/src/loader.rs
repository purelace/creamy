use std::{
    collections::HashMap,
    ffi::OsString,
    io::Cursor,
    path::PathBuf,
    sync::{Arc, atomic::AtomicUsize},
};

use async_zip::tokio::read::seek::ZipFileReader;
use inotify::{Event, EventMask};
use tokio::{
    io::AsyncReadExt,
    sync::{RwLock, Semaphore},
};

use crate::{
    PACKAGE_FILE_EXTENSION, config::CreamyConfig, progress::ProgressReader, watcher::FileWatcher,
};

pub enum LoadedCore {
    Directory,
    File(Vec<u8>),
}

pub struct LoadedPackage {
    manifest: String,
    core: LoadedCore,
    proto: Option<Vec<u8>>,
}

impl LoadedPackage {
    pub async fn new(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let cursor = Cursor::new(data);
        let mut reader = ZipFileReader::with_tokio(cursor).await?;
        let map = reader
            .file()
            .entries()
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.filename().clone().into_string().unwrap(), index))
            .collect::<HashMap<_, _>>();

        let manifest_index = map.get("manifest.toml").unwrap();
        let mut manifest_reader = reader.reader_with_entry(*manifest_index).await?;
        let mut manifest =
            String::with_capacity(manifest_reader.entry().uncompressed_size() as usize);
        manifest_reader
            .read_to_string_checked(&mut manifest)
            .await?;

        let proto = if let Some(proto_index) = map.get("proto.bin") {
            let mut proto_reader = reader.reader_with_entry(*proto_index).await?;
            let mut proto = Vec::with_capacity(proto_reader.entry().uncompressed_size() as usize);
            proto_reader.read_to_end_checked(&mut proto).await?;
            Some(proto)
        } else {
            None
        };

        if let Some(x) = map.get("core") {}

        println!("Loaded with manifest:  {manifest}");

        Ok(Self {
            manifest,
            core: LoadedCore::Directory,
            proto,
        })
    }
}

pub struct LoadingInProgress {
    progress: Arc<AtomicUsize>,
    data: Vec<u8>,
    is_done: bool,
}

pub struct PluginLoader {
    watcher: FileWatcher,
    directory: PathBuf,
    semaphore: Arc<Semaphore>,
    in_progress: HashMap<String, Arc<RwLock<LoadingInProgress>>>,
    loaded: HashMap<String, LoadedPackage>,
}

impl PluginLoader {
    pub fn new(config: CreamyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            semaphore: Arc::new(Semaphore::new(config.parallel_downloads())),
            watcher: FileWatcher::new(config.plugin_directory())?,
            directory: config.plugin_directory().into(),
            in_progress: HashMap::new(),
            loaded: HashMap::new(),
        })
    }

    async fn load_file(&mut self, file: PathBuf) {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();

        tracing::info!("--- Loading package: {}", file.display());
        let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
        let mut reader = ProgressReader::from_file(file).await;

        let state = Arc::new(RwLock::new(LoadingInProgress {
            progress: reader.counter(),
            data: Vec::with_capacity(reader.total()),
            is_done: false,
        }));

        assert!(self.in_progress.insert(file_name, state.clone()).is_none());

        tokio::spawn(async move {
            let mut state_write = state.write().await;
            reader.read_to_end(&mut state_write.data).await.unwrap();
            state_write.is_done = true;

            //tracing::info!("--- File loaded: {}", file.display());
            drop(permit);

            LoadedPackage::new(&state_write.data).await.unwrap();
        });
    }

    async fn validate_event(&self, event: Event<OsString>) -> Option<PathBuf> {
        match event.mask {
            EventMask::DELETE => return None,
            EventMask::MOVED_FROM => return None,
            EventMask::MOVED_TO => return None,
            _ => {}
        };

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

    pub async fn run(&mut self) {
        loop {
            let Some(event) = self.watcher.run_file_watcher().await else {
                continue;
            };

            //dbg!(&event);

            let Some(path) = self.validate_event(event).await else {
                return;
            };

            self.load_file(path).await;
        }
    }
}

// че сделать:
// решить вопрос с импортов/экспортов буферов.
// решить вопрос с переполнением буферов.
// исправить все unwrap/expect
// сделать загрузчик .cmy файлов.
// сделать возможность предоставления прогресса. (progress bar)
// в прогресс входит: чтение с диска, распаковка, загрузка схем,
// загрузка proto.bin (если есть), загрузка ядра и инициализация всего этого добра
//
//
// на будущее:
// работа со скриптами без .cmy файлов.
