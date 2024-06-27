use std::str::FromStr;

use axum::extract::{Path, State};
use blake3::Hash;
use bytes::Bytes;

use crate::{serve::BuildServerState, storage::HashStorage, BuilderError};

#[utoipa::path(
  get,
  path = "/api/v0/module/{id}",
  responses(
    (status = 200, description = "Successfully retrieved the module", body = Vec<u8>),
    (status = 404, description = "Module not found", body = ErrorResponse),
  )
)]
pub async fn retrieve_module(
    State(BuildServerState { storage, .. }): State<BuildServerState>,
    Path((id,)): Path<(String,)>,
) -> Result<Bytes, BuilderError> {
    let hash = Hash::from_str(&id)?;

    match storage.read(&hash).await? {
        Some(wasm) => Ok(wasm),
        _ => Err(BuilderError::ModuleNotFound),
    }
}
