use std::str::FromStr;

use crate::{
    error::BuilderError,
    storage::{HashStorage, PersistedHashStorage},
    Bake, Baker, JavaScriptBundler,
};
use async_trait::async_trait;
use blake3::Hash;
use common_protos::{
    builder::{
        builder_server::{Builder as BuilderProto, BuilderServer},
        BuildComponentRequest, BuildComponentResponse, BundleSourceCodeRequest,
        BundleSourceCodeResponse, ReadComponentRequest, ReadComponentResponse,
    },
    common::{ContentType, ModuleSource, Target as TargetProto},
    MAX_MESSAGE_SIZE,
};
use common_wit::Target;
use tokio::net::TcpListener;
use tonic::{transport::Server as TonicServer, Request, Response, Status};

pub struct Builder {
    storage: PersistedHashStorage,
}

impl Builder {
    fn take_one_from_module_source(
        &self,
        module_source: Option<ModuleSource>,
    ) -> Option<(Target, ContentType, Vec<u8>)> {
        if let Some(module_source) = module_source {
            let target = match module_source.target() {
                TargetProto::CommonFunction => Target::CommonFunction,
                TargetProto::CommonFunctionVm => Target::CommonFunctionVm,
            };
            let source_code = module_source.source_code;

            if source_code.len() > 1 {
                // TODO: Support multiple source inputs
                warn!("Multiple sources were provided, but only one will be used!")
            }

            if let Some((_, source_code)) = source_code.into_iter().next() {
                Some((target, source_code.content_type(), source_code.body))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl BuilderProto for Builder {
    async fn build_component(
        &self,
        request: Request<BuildComponentRequest>,
    ) -> Result<Response<BuildComponentResponse>, Status> {
        let request = request.into_inner();

        let (target, baker, source_code) = if let Some((target, content_type, source_code)) =
            self.take_one_from_module_source(request.module_source)
        {
            let baker = match content_type {
                ContentType::JavaScript => Baker::JavaScript,
                ContentType::Python => Baker::Python,
            };
            (target, baker, source_code)
        } else {
            return Err(Status::invalid_argument(
                "Must provide at least one source to build",
            ));
        };

        let bytes = baker.bake(target, source_code.into()).await?;
        let hash = self.storage.write(bytes).await?;

        Ok(Response::new(BuildComponentResponse {
            id: hash.to_string(),
        }))
    }

    async fn read_component(
        &self,
        request: Request<ReadComponentRequest>,
    ) -> Result<Response<ReadComponentResponse>, Status> {
        let request = request.into_inner();
        let hash = Hash::from_str(&request.id)
            .map_err(|error| Status::invalid_argument(format!("Could not parse ID: {error}")))?;

        Ok(Response::new(ReadComponentResponse {
            component: self
                .storage
                .read(&hash)
                .await?
                .ok_or(BuilderError::ModuleNotFound)?
                .to_vec(),
        }))
    }

    async fn bundle_source_code(
        &self,
        request: Request<BundleSourceCodeRequest>,
    ) -> Result<Response<BundleSourceCodeResponse>, Status> {
        let request = request.into_inner();

        let source_code = if let Some((target, content_type, source_code)) =
            self.take_one_from_module_source(request.module_source)
        {
            match (target, content_type) {
                (Target::CommonFunction, ContentType::JavaScript) => source_code,
                _ => {
                    return Err(Status::invalid_argument(
                        "Only JavaScript targetting 'common:module' may be bundled!",
                    ))
                }
            }
        } else {
            return Err(Status::invalid_argument(
                "Must provide at least one source to bundle",
            ));
        };

        let bundled_source_code =
            JavaScriptBundler::bundle_from_bytes_sync(source_code.into()).await?;

        Ok(Response::new(BundleSourceCodeResponse {
            bundled_source_code,
        }))
    }
}

impl From<BuilderError> for Status {
    fn from(value: BuilderError) -> Self {
        match value {
            BuilderError::BadRequest => Status::invalid_argument("Request was malformed"),
            BuilderError::InvalidConfiguration(error) => Status::failed_precondition(error),
            BuilderError::InvalidModule(error) => Status::invalid_argument(error),
            BuilderError::ModuleNotFound => Status::internal("Module not found"),
            BuilderError::Internal(error) => Status::internal(error),
        }
    }
}

/// Start the Common Builder server, listening to incoming connections on the
/// provided [TcpListener]
pub async fn serve(listener: TcpListener) -> Result<(), BuilderError> {
    let incoming_stream = async_stream::stream! {
        loop {
            let (stream, _) = listener.accept().await?;
            yield Ok::<_, std::io::Error>(stream);
        }
    };

    let storage = PersistedHashStorage::temporary()?;
    let builder_server = BuilderServer::new(Builder { storage })
        .max_encoding_message_size(MAX_MESSAGE_SIZE)
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

    TonicServer::builder()
        .add_service(builder_server)
        .serve_with_incoming(incoming_stream)
        .await
        .map_err(|error| BuilderError::Internal(format!("Failed to start server: {error}")))?;

    Ok(())
}
