use std::path::{Path, PathBuf};

use creamy_engine_core::PACKAGE_FILE_EXTENSION;
use inotify::{EventMask, Events, Inotify, WatchDescriptor, WatchMask};

pub struct FileIterator<'a> {
    directory: PathBuf,
    iter: Events<'a>,
}

impl Iterator for FileIterator<'_> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.iter.next()?;

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
            && match std::fs::metadata(&path) {
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
}

pub struct FileWatcher {
    _descriptor: WatchDescriptor,
    directory: PathBuf,
    inotify: Inotify,
    buffer: Vec<u8>,
}

impl FileWatcher {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let directory = directory.as_ref();
        let inotify = Inotify::init()?;
        let descriptor = inotify.watches().add(
            directory,
            WatchMask::DELETE | WatchMask::CLOSE_WRITE | WatchMask::MOVE,
        )?;
        Ok(Self {
            _descriptor: descriptor,
            directory: directory.into(),
            inotify,
            buffer: vec![0u8; 4096],
        })
    }

    pub fn try_read_events<'a>(&'a mut self) -> Result<FileIterator<'a>, std::io::Error> {
        Ok(FileIterator {
            directory: self.directory.clone(),
            iter: self.inotify.read_events(&mut self.buffer)?,
        })
    }
}
