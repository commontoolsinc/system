use crate::{artifact::Artifact, Error};
use async_trait::async_trait;
use redb::{Database, TableDefinition};
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Key-type for [`HashStorage`].
pub type Hash = blake3::Hash;

/// Trait for reading and writing bytes keyed by a [Hash].
#[async_trait]
pub trait JsComponentStorage: Send + Sync {
    /// Read value stored at `key`.
    async fn read(&self, key: &Hash) -> Result<Option<Artifact>, Error>;
    /// Write `value` to `key`.
    async fn write(&self, value: Artifact) -> Result<Hash, Error>;
}

const MODULE_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("modules");
const SOURCE_MAP_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("source_maps");

/// Simple key-value storage that persists to disk.
#[derive(Clone)]
pub struct PersistedHashStorage {
    db: Arc<Database>,
    _temp_file: Option<Arc<NamedTempFile>>,
}

impl PersistedHashStorage {
    /// Create a new [PersistedHashStorage] backed by a temporary directory.
    pub fn temporary() -> Result<Self, Error> {
        info!("Initializing temporary storage");
        let temp_file = Arc::new(NamedTempFile::new()?);
        let db = Arc::new(Database::create(temp_file.path())?);
        {
            // Create tables upfront, as opening a table
            // from a ReadTransaction errors if table
            // does not exist.
            let tx = db.begin_write()?;
            let _ = tx.open_table(MODULE_TABLE)?;
            let _ = tx.open_table(SOURCE_MAP_TABLE)?;
            tx.commit()?;
        }

        Ok(Self {
            db,
            _temp_file: Some(temp_file),
        })
    }
}

#[async_trait]
impl JsComponentStorage for PersistedHashStorage {
    async fn read(&self, key: &Hash) -> Result<Option<Artifact>, Error> {
        info!(?key, "Read");
        let key = key.to_string();
        let tx = self.db.begin_read()?;

        let component = {
            let table = tx.open_table(MODULE_TABLE)?;
            match table.get(key.as_str())?.map(|v| v.value()) {
                Some(component) => String::from_utf8(component)?,
                None => return Ok(None),
            }
        };

        let source_map = {
            let table = tx.open_table(SOURCE_MAP_TABLE)?;
            match table.get(key.as_str())?.map(|v| v.value()) {
                Some(source_map) => Some(String::from_utf8(source_map)?),
                None => None,
            }
        };

        Ok(Some(Artifact {
            component,
            source_map,
        }))
    }

    async fn write(&self, value: Artifact) -> Result<Hash, Error> {
        let component_bytes = value.component.into_bytes();
        let component_id = blake3::hash(&component_bytes);
        let key = component_id.to_string();
        let tx = self.db.begin_write()?;
        {
            let mut table = tx.open_table(MODULE_TABLE)?;
            table.insert(key.as_str(), component_bytes)?;
        }

        let source_map_bytes = value.source_map.map(|s| s.into_bytes());
        if let Some(source_map_bytes) = source_map_bytes {
            let mut table = tx.open_table(SOURCE_MAP_TABLE)?;
            table.insert(key.as_str(), source_map_bytes)?;
        }

        tx.commit()?;

        Ok(component_id)
    }
}
