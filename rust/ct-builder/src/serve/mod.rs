use crate::{builder::Builder, error::Error, import_map::ImportMap, storage::PersistedHashStorage};
use grpc::serve_grpc;
use http::serve_http;
use std::path::PathBuf;
use tokio::net::TcpListener;

mod grpc;
mod http;

#[derive(Default)]
pub struct BuildServerConfig {
    grpc_listener: Option<TcpListener>,
    http_listener: Option<TcpListener>,
    import_map: Option<ImportMap>,
}

impl BuildServerConfig {
    pub fn with_grpc(mut self, listener: TcpListener) -> Self {
        self.grpc_listener = Some(listener);
        self
    }

    pub fn with_http(mut self, listener: TcpListener) -> Self {
        self.http_listener = Some(listener);
        self
    }

    pub fn with_import_map(mut self, import_map: ImportMap) -> Self {
        self.import_map = Some(import_map);
        self
    }
}

/// Start the Common Builder server, serving gRPC on `grpc_listener`.
pub async fn serve(mut config: BuildServerConfig) -> Result<(), Error> {
    let grpc_listener = config.grpc_listener.take();
    let http_listener = config.http_listener.take();
    let import_map = config.import_map.take();

    let storage = PersistedHashStorage::temporary()?;
    let builder = Builder::new(storage);

    match (grpc_listener, http_listener) {
        (Some(grpc_listener), Some(http_listener)) => tokio::select! {
            _ = serve_grpc(builder.clone(), grpc_listener) => {},
            _ = serve_http(builder, http_listener, import_map) => {},
        },
        (Some(grpc_listener), None) => serve_grpc(builder, grpc_listener).await?,
        (None, Some(http_listener)) => serve_http(builder, http_listener, import_map).await?,
        (None, None) => return Err(Error::Internal("No HTTP or GRPC ports to run.".into())),
    }
    Ok(())
}
