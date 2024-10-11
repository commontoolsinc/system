use super::LiveModules;
use crate::NativeRuntime;
use crate::{
    formula::{instantiate_formula, run_end_formula, run_init_formula, run_step_formula},
    run::run_module,
    serve::instantiate::instantiate_module,
    ArtifactResolver, CommonRuntimeError,
};
use async_trait::async_trait;
use common_protos::{
    formula::{
        InstantiateFormulaRequest, InstantiateFormulaResponse, RunEndFormulaRequest,
        RunEndFormulaResponse, RunInitFormulaRequest, RunInitFormulaResponse,
        RunStepFormulaRequest, RunStepFormulaResponse,
    },
    runtime::{
        runtime_server::{Runtime as RuntimeServerHandlers, RuntimeServer},
        InstantiateModuleRequest, InstantiateModuleResponse, RunModuleRequest, RunModuleResponse,
    },
    MAX_MESSAGE_SIZE,
};
use http::{HeaderName, Uri};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex};
use tonic::{transport::Server as TonicServer, Status};
use tonic_web::GrpcWebLayer;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

/// A server-side entrypoint for sandboxed module instantiation
pub struct Server {
    runtime: Arc<Mutex<NativeRuntime>>,
    live_modules: Arc<Mutex<LiveModules>>,
}

impl Server {
    /// Instantiate a new[`Server`]; the optional `builder_address` will be used
    /// to attempt to JIT prepare not-yet-compiled Common Modules when needed.
    pub fn new(builder_address: Option<Uri>) -> Result<Self, CommonRuntimeError> {
        let artifact_resolver = ArtifactResolver::new(builder_address.clone())?;
        let runtime = NativeRuntime::new(artifact_resolver)?;

        Ok(Server {
            runtime: Arc::new(Mutex::new(runtime)),
            live_modules: Arc::new(Mutex::new(LiveModules::default())),
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
                self.live_modules.clone(),
            )
            .await?,
        ))
    }

    async fn run_module(
        &self,
        request: tonic::Request<RunModuleRequest>,
    ) -> Result<tonic::Response<RunModuleResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            run_module(request.into_inner(), self.live_modules.clone()).await?,
        ))
    }

    async fn instantiate_formula(
        &self,
        request: tonic::Request<InstantiateFormulaRequest>,
    ) -> Result<tonic::Response<InstantiateFormulaResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            instantiate_formula(
                request.into_inner(),
                self.runtime.clone(),
                self.live_modules.clone(),
            )
            .await?,
        ))
    }

    async fn run_init_formula(
        &self,
        request: tonic::Request<RunInitFormulaRequest>,
    ) -> Result<tonic::Response<RunInitFormulaResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            run_init_formula(request.into_inner(), self.live_modules.clone()).await?,
        ))
    }

    async fn run_step_formula(
        &self,
        request: tonic::Request<RunStepFormulaRequest>,
    ) -> Result<tonic::Response<RunStepFormulaResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            run_step_formula(request.into_inner(), self.live_modules.clone()).await?,
        ))
    }

    async fn run_end_formula(
        &self,
        request: tonic::Request<RunEndFormulaRequest>,
    ) -> Result<tonic::Response<RunEndFormulaResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            run_end_formula(request.into_inner(), self.live_modules.clone()).await?,
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
            CommonRuntimeError::InvalidModuleSource(_) => {
                Status::invalid_argument(format!("{value}"))
            }
            CommonRuntimeError::InvalidInstantiationParameters(_) => {
                Status::invalid_argument(format!("{value}"))
            }
            CommonRuntimeError::PolicyRejection(_) => Status::invalid_argument(format!("{value}")),
            CommonRuntimeError::InvalidValueKind(_) => Status::invalid_argument(format!("{value}")),
        }
    }
}

const DEFAULT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const DEFAULT_EXPOSED_HEADERS: [&str; 3] =
    ["grpc-status", "grpc-message", "grpc-status-details-bin"];
const DEFAULT_ALLOW_HEADERS: [&str; 4] =
    ["x-grpc-web", "content-type", "x-user-agent", "grpc-timeout"];

/// Start the Common Runtime server, listening to incoming connections on the
/// provided[`TcpListener`]
pub async fn serve(
    listener: TcpListener,
    builder_address: Option<Uri>,
) -> Result<(), CommonRuntimeError> {
    let incoming_stream = async_stream::stream! {
        loop {
            let (stream, _) = listener.accept().await?;
            yield Ok::<_, std::io::Error>(stream);
        }
    };

    let runtime_server = RuntimeServer::new(Server::new(builder_address)?)
        .max_encoding_message_size(MAX_MESSAGE_SIZE)
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

    TonicServer::builder()
        .accept_http1(true)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::mirror_request())
                .allow_credentials(true)
                .max_age(DEFAULT_MAX_AGE)
                .expose_headers(
                    DEFAULT_EXPOSED_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                )
                .allow_headers(
                    DEFAULT_ALLOW_HEADERS
                        .iter()
                        .cloned()
                        .map(HeaderName::from_static)
                        .collect::<Vec<HeaderName>>(),
                ),
        )
        .layer(GrpcWebLayer::new())
        .add_service(runtime_server)
        .serve_with_incoming(incoming_stream)
        .await
        .map_err(|error| {
            CommonRuntimeError::InternalError(format!("Failed to start server: {error}"))
        })?;

    Ok(())
}
