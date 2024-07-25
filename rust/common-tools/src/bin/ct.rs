#[cfg(target_arch = "wasm32")]
pub fn main() {
    unimplemented!("Binary not supported for wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use common_tools::{exec_command, Cli};
    use tracing_subscriber::{EnvFilter, FmtSubscriber};
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to configure tracing");

    exec_command(Cli::parse()).await
}
