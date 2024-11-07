use crate::{Error, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ImportMap(HashMap<String, PathBuf>);

impl ImportMap {
    pub fn get(&self, key: &str) -> Option<&PathBuf> {
        self.0.get(key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, PathBuf> {
        self.0.iter()
    }

    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path: &Path = path.as_ref();
        if path.is_relative() {
            return Err(Error::Internal(
                "ImportMap must be constructed from an absolute path.".into(),
            ));
        }
        let json_str = tokio::fs::read_to_string(path).await?;
        let import_map_dir = path
            .parent()
            .ok_or_else(|| Error::Internal("No parent dir found.".into()))?;
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;
        let Some(js_map) = parsed["imports"].as_object() else {
            return Err(Error::Internal("\"imports\" is not an object.".into()));
        };

        let mut map = HashMap::default();
        for (key, value) in js_map.iter() {
            let Some(value) = value.as_str() else {
                return Err(Error::Internal(format!("Path for {} not a string.", key)));
            };
            let abs_path = {
                let value_path = PathBuf::from(value);
                if value_path.is_relative() {
                    import_map_dir.join(value_path).canonicalize()?
                } else {
                    value_path
                }
            };
            map.insert(key.to_owned(), abs_path);
        }

        println!("imports_map {:#?}", map);
        Ok(ImportMap(map))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ct_tracing::ct_tracing;

    #[tokio::test]
    #[ct_tracing]
    async fn it_loads_import_maps_from_disk() -> Result<()> {
        let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let import_map_path = fixtures_dir.join("imports.json");
        let import_map = ImportMap::from_path(import_map_path).await?;

        assert_eq!(
            import_map.get("test:math").unwrap().to_string_lossy(),
            fixtures_dir.join("test/math/index.js").to_string_lossy()
        );
        Ok(())
    }
}
