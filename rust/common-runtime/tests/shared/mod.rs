use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_protos::runtime::runtime_client::RuntimeClient;
use common_runtime::{serve as serve_runtime, CommonRuntimeError};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Starts a `common-runtime` server connected to a `common-builder`,
/// returning a handle to a [RuntimeClient], and handlers to
/// tokio threads for the server lifetimes.
pub async fn start_runtime() -> Result<(
    RuntimeClient<tonic::transport::channel::Channel>,
    JoinHandle<Result<(), CommonRuntimeError>>,
    JoinHandle<Result<(), BuilderError>>,
)> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address: http::Uri =
        format!("http://{}", builder_listener.local_addr()?).parse()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener, Some(builder_address)));

    let runtime_client = RuntimeClient::connect(format!("http://{}", runtime_address)).await?;

    Ok((runtime_client, runtime_task, builder_task))
}
