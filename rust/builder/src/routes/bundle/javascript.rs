use axum::extract::Multipart;
use bundler::JavaScriptBundler;
use utoipa::ToSchema;

use crate::BuilderError;

/// Parameters used in a request to the bundler service.
#[derive(ToSchema)]
pub struct BundleRequest {
    /// Collection of JavaScript source strings.
    #[allow(unused)]
    pub source: Vec<Vec<u8>>,
}

#[utoipa::path(
  post,
  path = "/api/v0/bundle",
  request_body(content = BundleRequest, content_type = "multipart/form-data"),
  responses(
    (status = 200, description = "Successfully built the module", body = String, content_type = "text/javascript"),
    (status = 400, description = "Bad request body", body = ErrorResponse),
    (status = 500, description = "Internal error", body = ErrorResponse)
  )
)]
pub async fn bundle_javascript(mut form_data: Multipart) -> Result<String, BuilderError> {
    let first_field = if let Some(field) = form_data.next_field().await? {
        field
    } else {
        return Err(BuilderError::BadRequest);
    };

    match first_field.name() {
        Some("source") => match first_field.file_name() {
            Some(name) if name.ends_with(".js") => {
                let source_code = first_field.bytes().await?;
                return Ok(tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current()
                        .block_on(JavaScriptBundler::bundle_module(source_code))
                })
                .await??);
            }
            _ => warn!("Skipping unexpected content type"),
        },
        _ => warn!("Skipping unexpected multipart content"),
    }

    Err(BuilderError::BadRequest)
}
