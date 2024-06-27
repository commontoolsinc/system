use std::path::PathBuf;

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wit_parser::UnresolvedPackageGroup;

use crate::{serve::BuildServerState, storage::HashStorage, Bake, Baker, BuilderError};

/// A `multipart/form-data` payload that consists of module WIT + source code as
/// well as additional (optional) library WIT files
#[derive(ToSchema)]
pub struct BuildModuleRequest {
    /// Collection of module WIT + source code.
    #[allow(unused)]
    pub module: Vec<Vec<u8>>,
    /// Collection of optional library WIT source.
    #[allow(unused)]
    pub library: Vec<Vec<u8>>,
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
    let mut world_name: Option<String> = None;
    let mut wit: Vec<Bytes> = Vec::new();
    let mut library: Vec<Bytes> = Vec::new();
    let mut source_code: Option<Bytes> = None;
    let mut baker: Option<Baker> = None;

    while let Some(field) = form_data.next_field().await? {
        if let Some(file_name) = field.file_name() {
            let file_name = PathBuf::from(file_name);

            match field.name() {
                Some("module") => {
                    if let Some(extension) = file_name.extension() {
                        match extension.to_str() {
                            Some("wit") => {
                                let module_wit = field.bytes().await?;
                                let package_group = UnresolvedPackageGroup::parse(
                                    &PathBuf::from("module.wit"),
                                    String::from_utf8_lossy(&module_wit).as_ref(),
                                )?;

                                let wit_package =
                                    package_group.packages.first().ok_or_else(|| {
                                        BuilderError::InvalidConfiguration(
                                            "Malformed module.wit".into(),
                                        )
                                    })?;

                                wit.push(module_wit);

                                world_name = Some(
                                    wit_package
                                        .worlds
                                        .iter()
                                        .nth(0)
                                        .map(|(_, world)| world.name.clone())
                                        .ok_or_else(|| {
                                            BuilderError::InvalidModule(format!(
                                                "Module WIT does not contain a world"
                                            ))
                                        })?,
                                );
                            }
                            Some("js") => {
                                source_code = Some(field.bytes().await?);
                                baker = Some(Baker::JavaScript);
                            }
                            Some("py") => {
                                source_code = Some(field.bytes().await?);
                                baker = Some(Baker::Python);
                            }
                            _ => (),
                        };
                    }
                }
                Some("library") => {
                    library.push(field.bytes().await?);
                }
                Some(name) => warn!("Unexpected multipart content: {name}"),
                _ => warn!("Skipping unnamed multipart content"),
            };
        }
    }

    if let (Some(world_name), Some(source_code), Some(baker)) = (world_name, source_code, baker) {
        let wasm = baker.bake(&world_name, wit, source_code, library).await?;
        let hash = storage.write(wasm).await?;

        Ok(BuildModuleResponse {
            id: hash.to_string(),
        })
    } else {
        warn!("Insufficient payload inputs to build the module");
        Err(BuilderError::BadRequest)
    }
}
