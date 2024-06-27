#[macro_use]
extern crate tracing;

use std::net::SocketAddr;

use builder::{serve, BuilderError};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, EnvFilter, FmtSubscriber};

#[tokio::main]
pub async fn main() -> Result<(), BuilderError> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber.with(Layer::default().pretty()))?;

    let port = std::env::var("PORT").unwrap_or("8081".into());
    let socket_address: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    info!("Server listening on {}", socket_address);

    serve(listener).await?;

    Ok(())
}
