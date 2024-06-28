#[macro_use]
extern crate tracing;

use common_builder::{serve, BuilderError};
use std::net::SocketAddr;

#[tokio::main]
pub async fn main() -> Result<(), BuilderError> {
    common_tracing::initialize_tracing();

    let port = std::env::var("PORT").unwrap_or("8081".into());
    let socket_address: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    info!("Server listening on {}", socket_address);

    serve(listener).await?;

    Ok(())
}
