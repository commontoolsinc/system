use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    error::BuilderError,
    openapi::OpenApiDocs,
    routes::{build_module, bundle_javascript, retrieve_module},
    storage::PersistedHashStorage,
};

#[derive(Clone)]
pub struct BuildServerState {
    pub storage: PersistedHashStorage,
}

/// Start the build server with `listener`.
pub async fn serve(listener: TcpListener) -> Result<(), BuilderError> {
    let storage = PersistedHashStorage::temporary()?;

    let cors = CorsLayer::new()
        .allow_methods([Method::HEAD, Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", OpenApiDocs::openapi()))
        .route("/api/v0/bundle", post(bundle_javascript))
        .route("/api/v0/module", post(build_module))
        .route("/api/v0/module/:id", get(retrieve_module))
        .with_state(BuildServerState { storage })
        .layer(cors);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
