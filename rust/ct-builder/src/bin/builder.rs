#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate tracing;

#[cfg(target_arch = "wasm32")]
pub fn main() {
    unimplemented!("Binary not supported for wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> Result<(), ct_builder::Error> {
    use clap::Parser;
    use ct_builder::serve;
    use std::net::SocketAddr;
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    #[derive(clap::Parser)]
    #[command(version, about, long_about = None)]
    struct Cli {
        /// Set build server to listen on provided port.
        #[arg(short, long, default_value_t = 8082)]
        port: u16,
    }

    let cli = Cli::parse();
    let port = cli.port;
    let socket_address: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    info!("Server listening on {}", socket_address);

    serve(listener).await?;

    Ok(())
}
