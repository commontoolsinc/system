use std::sync::Arc;

use async_trait::async_trait;
use blake3::Hash;
use bytes::Bytes;
use redb::{Database, TableDefinition};
use tempfile::NamedTempFile;

use crate::BuilderError;

/// Trait for reading and writing bytes keyed by a [Hash].
#[async_trait]
pub trait HashStorage: Send + Sync {
    /// Read value stored at `key`.
    async fn read(&self, key: &Hash) -> Result<Option<Bytes>, BuilderError>;
    /// Write `value` to `key`.
    async fn write(&mut self, value: Bytes) -> Result<Hash, BuilderError>;
}

const MODULE_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("modules");

/// Simple key-value storage that persists to disk.
#[derive(Clone)]
pub struct PersistedHashStorage {
    db: Arc<Database>,
    _temp_file: Option<Arc<NamedTempFile>>,
}

impl PersistedHashStorage {
    /// Create a new [PersistedHashStorage] backed by a temporary directory.
    pub fn temporary() -> Result<Self, BuilderError> {
        let temp_file = Arc::new(NamedTempFile::new()?);
        let db = Arc::new(Database::create(temp_file.path())?);

        Ok(Self {
            db,
            _temp_file: Some(temp_file),
        })
    }
}

#[async_trait]
impl HashStorage for PersistedHashStorage {
    async fn read(&self, key: &Hash) -> Result<Option<Bytes>, BuilderError> {
        let tx = self.db.begin_read()?;
        let table = tx.open_table(MODULE_TABLE)?;

        Ok(table
            .get(key.to_string().as_str())?
            .map(|v| v.value().into()))
    }

    async fn write(&mut self, value: Bytes) -> Result<Hash, BuilderError> {
        let hash = blake3::hash(&value);

        let tx = self.db.begin_write()?;

        {
            let mut table = tx.open_table(MODULE_TABLE)?;
            table.insert(hash.to_string().as_str(), value.to_vec())?;
        }

        tx.commit()?;

        Ok(hash)
    }
}
