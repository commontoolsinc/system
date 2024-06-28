use crate::BuilderError;
use bytes::Bytes;
use std::{io::Cursor, path::PathBuf};

/// Utility to write bytes to a file.
pub async fn write_file(path: PathBuf, bytes: Bytes) -> Result<(), BuilderError> {
    let mut file = tokio::fs::File::create(&path).await?;
    let mut cursor = Cursor::new(bytes.as_ref());
    tokio::io::copy(&mut cursor, &mut file).await?;
    Ok(())
}
