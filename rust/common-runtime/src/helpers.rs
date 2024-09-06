//! Helpers for the Common Runtime.

use crate::{serve as serve_runtime, CommonRuntimeError};
use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_protos::runtime::runtime_client::RuntimeClient;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// The critical metadata needed to access the virtual environment
/// that is started by the [start_runtime] helper
pub struct VirtualEnvironment {
    /// A gRPC client that has been configured to access the running
    /// [crate::NativeRuntime]
    pub runtime_client: RuntimeClient<tonic::transport::channel::Channel>,
    /// The port of the gRPC server in front of the running
    /// [crate::NativeRuntime]
    pub runtime_port: u16,
    /// The port of the gRPC server on the running build server
    pub builder_port: u16,
    /// A [JoinHandle] for the Tokio task of the Runtime server
    pub runtime_task: JoinHandle<Result<(), CommonRuntimeError>>,
    /// A [JoinHandle] for the Tokio task of the Builder server
    pub builder_task: JoinHandle<Result<(), BuilderError>>,
}

/// Starts a `common-runtime` server connected to a `common-builder`,
/// returning a handle to a [RuntimeClient], and handlers to
/// tokio threads for the server lifetimes.
pub async fn start_runtime() -> Result<VirtualEnvironment> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_port = builder_listener.local_addr()?.port();
    let builder_address: http::Uri =
        format!("http://{}", builder_listener.local_addr()?).parse()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_port = runtime_address.port();
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener, Some(builder_address)));

    let runtime_client = RuntimeClient::connect(format!("http://{}", runtime_address)).await?;

    Ok(VirtualEnvironment {
        runtime_client,
        runtime_port,
        builder_port,
        runtime_task,
        builder_task,
    })
}
