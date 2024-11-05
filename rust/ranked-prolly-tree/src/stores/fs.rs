use crate::{BlockStore, Hash, HashDisplay, HashRef, Result};
use async_trait::async_trait;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// A file-system [`BlockStore`] implementation.
#[derive(Clone)]
pub struct FileSystemStore {
    root_dir: PathBuf,
}

impl FileSystemStore {
    /// Creates a new [`FileSystemStore`] stored in `root_dir`.
    pub async fn new<S: AsRef<Path>>(root_dir: S) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_owned();
        tokio::fs::create_dir_all(&root_dir).await?;
        Ok(Self { root_dir })
    }

    /// Encodes the hash using [`HashDisplay`], a lower hex encoding
    /// of the hash bytes.
    fn get_path(&self, hash: Hash) -> Result<PathBuf> {
        Ok(self.root_dir.join(HashDisplay::from(hash).to_string()))
    }
}
#[async_trait]
impl BlockStore for FileSystemStore {
    async fn get_block(&self, hash: &HashRef) -> Result<Option<Vec<u8>>> {
        match tokio::fs::read(self.get_path(hash.to_owned())?).await {
            Ok(value) => Ok(Some(value)),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(e.into()),
            },
        }
    }

    async fn set_block(&mut self, hash: Hash, bytes: Vec<u8>) -> Result<()> {
        tokio::fs::write(self.get_path(hash)?, bytes).await?;
        Ok(())
    }
}
