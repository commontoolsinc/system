#[macro_use]
extern crate tracing;

use std::net::SocketAddr;

use common_runtime::{serve, CommonRuntimeError};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
pub async fn main() -> Result<(), CommonRuntimeError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to configure tracing");

    let port = std::env::var("PORT").unwrap_or("8081".into());
    let socket_address: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .unwrap_or_else(|_| panic!("Invalid port: {port}"));
    let listener = tokio::net::TcpListener::bind(socket_address)
        .await
        .expect("Failed to bind TCP listener");

    info!("Server listening on {}", socket_address);

    serve(listener).await?;

    Ok(())
}
