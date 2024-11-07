use crate::{builder::Builder, error::Error, BuildComponentConfig, ImportMap};
use axum::{body::Body, extract::State, http::StatusCode, routing::post, Router};
use ct_common::ModuleDefinition;
use ct_runtime::Runtime;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Clone)]
struct ServerState {
    pub builder: Builder,
    pub runtime: Arc<Mutex<Runtime>>,
    pub import_map: Option<ImportMap>,
}

pub async fn serve_http(
    builder: Builder,
    listener: TcpListener,
    import_map: Option<ImportMap>,
) -> Result<(), Error> {
    let runtime = Arc::new(Mutex::new(
        Runtime::new(|_input| Err("Unsupported".into()))
            .map_err(|e| Error::Internal(e.to_string()))?,
    ));
    let state = ServerState {
        runtime,
        builder,
        import_map,
    };
    let app = Router::new()
        .route("/recipe", post(handle_recipe_build))
        .with_state(state);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_recipe_build(
    State(state): State<ServerState>,
    body: Body,
) -> std::result::Result<String, StatusCode> {
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let input = String::from_utf8(bytes.to_vec()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let hash = state
        .builder
        .build(BuildComponentConfig {
            definition: ModuleDefinition::from(input),
            bundle_common_imports: false,
            import_map: state.import_map.clone(),
        })
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let js_artifact = state
        .builder
        .read(hash)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    info!("GOT ARTIFACT {}", js_artifact.component);
    let runtime = state.runtime.lock().await;
    let mut module = runtime
        .module(ModuleDefinition::from(js_artifact.component))
        .map_err(|_| {
            info!("Sandbox module creation failed");
            StatusCode::BAD_REQUEST
        })?;
    let mut instance = module.instantiate().map_err(|_| {
        info!("Instantiation failed.");
        StatusCode::BAD_REQUEST
    })?;
    let output = instance.run(None).map_err(|_| {
        info!("Run failure");
        StatusCode::BAD_REQUEST
    })?;
    match output {
        Some(output) => Ok(output),
        None => {
            info!("JS function does not return a stringifiable object.");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
