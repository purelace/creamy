use std::{
    path::Path,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
};

use tokio::{fs::File, io::AsyncRead};

pub struct ProgressReader<R> {
    inner: R,
    total: usize,
    counter: Arc<AtomicUsize>,
}

impl ProgressReader<File> {
    pub async fn from_file(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).await.unwrap();
        let metadata = file.metadata().await.unwrap();
        let total = metadata.len() as usize;

        let loaded = Arc::new(AtomicUsize::new(0));

        Self {
            inner: file,
            total,
            counter: loaded,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for ProgressReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let prev_len = buf.filled().len();
        let res = Pin::new(&mut self.inner).poll_read(cx, buf);

        if let Poll::Ready(Ok(())) = res {
            let read = buf.filled().len() - prev_len;
            self.counter.fetch_add(read, Ordering::Relaxed);
        }
        res
    }
}

impl<R> ProgressReader<R> {
    pub const fn total(&self) -> usize {
        self.total
    }

    pub fn loaded(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }

    pub fn counter(&self) -> Arc<AtomicUsize> {
        self.counter.clone()
    }
}
