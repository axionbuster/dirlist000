//! Testing out a VFS implementation that lists files in a directory

use anyhow::Context;

mod vfs {
    //! Generic VFS specification

    use std::path::PathBuf;
    use std::pin::Pin;

    use tokio_stream::Stream;

    /// A generic stream of PathBuf's.
    pub type PathStream = Pin<Box<dyn Stream<Item = anyhow::Result<PathBuf>> + Send>>;

    /// A generic VFS specification.
    #[async_trait::async_trait]
    pub trait Vfs: Send + Sync + 'static {
        /// List all files in a directory.
        async fn list_files(&self, path: &std::path::Path) -> PathStream;
    }
}

struct TokioVfs;

#[async_trait::async_trait]
impl vfs::Vfs for TokioVfs {
    async fn list_files(&self, path: &std::path::Path) -> vfs::PathStream {
        use tokio_stream::StreamExt;

        let read_dir = tokio::fs::read_dir(path).await.unwrap();
        let stream = tokio_stream::wrappers::ReadDirStream::new(read_dir);
        let stream = stream.map(|entry| entry.context("context").map(|x| x.path()));
        Box::pin(stream)
    }
}

#[tokio::main]
async fn main() {
    use tokio_stream::StreamExt;

    use vfs::Vfs;

    tracing_subscriber::fmt::init();

    let root = std::env::current_dir().unwrap();
    let vfs = TokioVfs;
    let mut stream = vfs.list_files(&root).await;

    while let Some(path) = stream.next().await {
        tracing::info!(?path);
    }

    tracing::info!("done");
}
