use std::{path::PathBuf, str::FromStr};

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use common_wit::WitTarget;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{serve::BuildServerState, storage::HashStorage, Bake, Baker, BuilderError};

/// A `multipart/form-data` payload that consists of module WIT + source code as
/// well as additional (optional) library WIT files
#[derive(ToSchema)]
pub struct BuildModuleRequest {
    /// The source code of the module
    #[allow(unused)]
    pub source: Vec<u8>,

    /// The WIT target that the module source code implements
    #[allow(unused)]
    pub target: WitTarget,
}

/// A response from a build module request containing the
/// ID for the built artifact.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct BuildModuleResponse {
    id: String,
}

impl IntoResponse for BuildModuleResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[utoipa::path(
  post,
  path = "/api/v0/module",
  request_body(content = BuildModuleRequest, content_type = "multipart/form-data"),
  responses(
    (status = 200, description = "Successfully built the module", body = BuildModuleResponse),
    (status = 400, description = "Bad request body", body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse)
  )
)]
pub async fn build_module(
    State(BuildServerState { mut storage, .. }): State<BuildServerState>,
    mut form_data: Multipart,
) -> Result<BuildModuleResponse, BuilderError> {
    let mut source_code: Option<Bytes> = None;
    let mut target: Option<WitTarget> = None;
    let mut bakery: Option<Baker> = None;

    while let Some(field) = form_data.next_field().await? {
        match field.name() {
            Some("source") => {
                let file_name = if let Some(file_name) = field.file_name() {
                    PathBuf::from(file_name)
                } else {
                    continue;
                };

                if let Some(extension) = file_name.extension() {
                    match extension.to_str() {
                        Some("js") => {
                            source_code = Some(field.bytes().await?);
                            bakery = Some(Baker::JavaScript);
                        }
                        Some("py") => {
                            source_code = Some(field.bytes().await?);
                            bakery = Some(Baker::Python);
                        }
                        _ => (),
                    };
                }
            }
            Some("target") => {
                target = Some(WitTarget::from_str(
                    &String::from_utf8(field.bytes().await?.to_vec()).map_err(|error| {
                        BuilderError::InvalidModule(format!("Could not parse target: {error}"))
                    })?,
                )?);
            }
            Some(name) => warn!("Unexpected multipart content: {name}"),
            _ => warn!("Skipping unnamed multipart content"),
        };
    }
    warn!(
        "{:?} {:?} {:?}",
        bakery.is_some(),
        target.is_some(),
        source_code.is_some()
    );

    if let (Some(bakery), Some(target), Some(source_code)) = (bakery, target, source_code) {
        let wasm = bakery.bake(target, source_code).await?;
        let hash = storage.write(wasm).await?;

        Ok(BuildModuleResponse {
            id: hash.to_string(),
        })
    } else {
        warn!("Insufficient payload inputs to build the module");
        Err(BuilderError::BadRequest)
    }
}
