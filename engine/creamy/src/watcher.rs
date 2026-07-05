use std::ffi::OsString;

use futures_util::TryStreamExt;
use inotify::{Event, EventStream, Inotify, WatchDescriptor, WatchMask};

pub struct FileWatcher {
    watch_descriptor: WatchDescriptor,
    inotify: EventStream<Vec<u8>>,
}

impl FileWatcher {
    pub fn new(directory: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let inotify = Inotify::init()?;
        let watch_descriptor = inotify.watches().add(
            directory,
            WatchMask::DELETE | WatchMask::CLOSE_WRITE | WatchMask::MOVE,
        )?;
        let inotify = inotify.into_event_stream(vec![0; 4096])?;
        Ok(Self {
            watch_descriptor,
            inotify,
        })
    }

    pub async fn run_file_watcher(&mut self) -> Option<Event<OsString>> {
        match self.inotify.try_next().await {
            Ok(result) => result,
            Err(error) => {
                eprintln!("File watcher error: {error}");
                None
            }
        }
    }
}
