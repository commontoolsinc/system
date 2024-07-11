use crate::{run::run_module, serve::instantiate::instantiate_module, CommonRuntimeError, Runtime};
use async_trait::async_trait;
use common_protos::{
    runtime::{
        runtime_server::{Runtime as RuntimeServerHandlers, RuntimeServer},
        InstantiateModuleRequest, InstantiateModuleResponse, RunModuleRequest, RunModuleResponse,
    },
    MAX_MESSAGE_SIZE,
};
use http::Uri;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};
use tonic::{transport::Server as TonicServer, Status};

/// A server-side entrypoint for sandboxed module instantiation
pub struct Server {
    builder_address: Option<Uri>,
    runtime: Arc<Mutex<Runtime>>,
}

impl Server {
    /// Instantiate a new [Server]; the optional `builder_address` will be used
    /// to attempt to JIT prepare not-yet-compiled Common Modules when needed.
    pub fn new(builder_address: Option<Uri>) -> Result<Self, CommonRuntimeError> {
        Ok(Server {
            builder_address,
            runtime: Arc::new(Mutex::new(Runtime::new()?)),
        })
    }
}

#[async_trait]
impl RuntimeServerHandlers for Server {
    async fn instantiate_module(
        &self,
        request: tonic::Request<InstantiateModuleRequest>,
    ) -> Result<tonic::Response<InstantiateModuleResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            instantiate_module(
                request.into_inner(),
                self.runtime.clone(),
                self.builder_address.clone(),
            )
            .await?,
        ))
    }

    async fn run_module(
        &self,
        request: tonic::Request<RunModuleRequest>,
    ) -> Result<tonic::Response<RunModuleResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            run_module(request.into_inner(), self.runtime.clone()).await?,
        ))
    }
}

impl From<CommonRuntimeError> for Status {
    fn from(value: CommonRuntimeError) -> Self {
        match value {
            CommonRuntimeError::PreparationFailed(_) => Status::aborted(format!("{value}")),
            CommonRuntimeError::LinkFailed(_) => Status::aborted(format!("{value}")),
            CommonRuntimeError::SandboxCreationFailed(_) => Status::internal(format!("{value}")),
            CommonRuntimeError::ModuleInstantiationFailed(_) => Status::aborted(format!("{value}")),
            CommonRuntimeError::ModuleRunFailed(_) => Status::aborted(format!("{value}")),
            CommonRuntimeError::InternalError(_) => Status::internal(format!("{value}")),
            CommonRuntimeError::InvalidValue => Status::invalid_argument(format!("{value}")),
            CommonRuntimeError::InvalidModuleId(_) => Status::invalid_argument(format!("{value}")),
            CommonRuntimeError::UnknownInstanceId(_) => {
                Status::invalid_argument(format!("{value}"))
            }
        }
    }
}

/// Start the Common Runtime server, listening to incoming connections on the
/// provided [TcpListener]
pub async fn serve(listener: TcpListener) -> Result<(), CommonRuntimeError> {
    let incoming_stream = async_stream::stream! {
        loop {
            let (stream, _) = listener.accept().await?;
            yield Ok::<_, std::io::Error>(stream);
        }
    };

    let builder_address = if let Ok(raw_uri) = std::env::var("BUILDER_ADDRESS") {
        raw_uri.parse().ok()
    } else {
        None
    };

    let runtime_server = RuntimeServer::new(Server::new(builder_address)?)
        .max_encoding_message_size(MAX_MESSAGE_SIZE)
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

    TonicServer::builder()
        .add_service(runtime_server)
        .serve_with_incoming(incoming_stream)
        .await
        .map_err(|error| {
            CommonRuntimeError::InternalError(format!("Failed to start server: {error}"))
        })?;

    Ok(())
}
