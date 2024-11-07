#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate tracing;

#[cfg(target_arch = "wasm32")]
pub fn main() {
    unimplemented!("Binary not supported for wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// GRPC server port.
    #[arg(short = 'g', long, default_value_t = 8082)]
    pub grpc_port: u16,
    /// HTTP server port.
    #[arg(short = 'p', long, default_value_t = 8081)]
    pub http_port: u16,
    /// HTTP server port.
    #[arg(short = 'i', long)]
    pub import_map: Option<std::path::PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> Result<(), ct_builder::Error> {
    use clap::Parser;
    use ct_builder::{serve, BuildServerConfig, ImportMap};
    use std::net::SocketAddr;
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    let grpc_listener = {
        let socket_address: SocketAddr = format!("0.0.0.0:{}", cli.grpc_port).parse()?;
        info!("GRPC listening on {}", socket_address);
        tokio::net::TcpListener::bind(socket_address).await?
    };
    let http_listener = {
        let socket_address: SocketAddr = format!("0.0.0.0:{}", cli.http_port).parse()?;
        info!("HTTP listening on {}", socket_address);
        tokio::net::TcpListener::bind(socket_address).await?
    };
    let mut config = BuildServerConfig::default()
        .with_grpc(grpc_listener)
        .with_http(http_listener);
    if let Some(import_map_path) = cli.import_map {
        let import_map_path = if import_map_path.is_relative() {
            std::env::current_dir()?
                .join(import_map_path)
                .canonicalize()?
        } else {
            import_map_path
        };
        let import_map = ImportMap::from_path(import_map_path).await?;
        config = config.with_import_map(import_map);
    }
    serve(config).await?;

    Ok(())
}
