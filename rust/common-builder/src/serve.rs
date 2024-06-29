use std::str::FromStr;

use async_trait::async_trait;
use blake3::Hash;
use bytes::Bytes;
use common_wit::WitTarget;
use tokio::net::TcpListener;
use tonic::{transport::Server as TonicServer, Request, Response, Status};

use crate::{
    error::BuilderError,
    proto::{
        builder_server::{Builder as BuilderProto, BuilderServer},
        BuildComponentRequest, BuildComponentResponse, BundleSourceCodeRequest,
        BundleSourceCodeResponse, ContentType, ReadComponentRequest, ReadComponentResponse, Target,
    },
    storage::{HashStorage, PersistedHashStorage},
    Bake, Baker, JavaScriptBundler,
};

pub struct Builder {
    storage: PersistedHashStorage,
}

#[async_trait]
impl BuilderProto for Builder {
    async fn build_component(
        &self,
        request: Request<BuildComponentRequest>,
    ) -> Result<Response<BuildComponentResponse>, Status> {
        let request = request.into_inner();

        let baker = match request.content_type() {
            ContentType::Javascript => Baker::JavaScript,
            ContentType::Python => Baker::Python,
        };

        let target = match request.target() {
            Target::CommonModule => WitTarget::CommonModule,
        };

        let bytes = baker.bake(target, Bytes::from(request.source_code)).await?;
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

        let bundled_source_code = JavaScriptBundler::bundle_sync(request.source_code).await?;

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

/// Start the Common Builder server, listening to incomming connections on the
/// provided [TcpListener]
pub async fn serve(listener: TcpListener) -> Result<(), BuilderError> {
    let incoming_stream = async_stream::stream! {
        loop {
            let (stream, _) = listener.accept().await?;
            yield Ok::<_, std::io::Error>(stream);
        }
    };

    let storage = PersistedHashStorage::temporary()?;
    let builder_server = BuilderServer::new(Builder { storage });

    TonicServer::builder()
        .add_service(builder_server)
        .serve_with_incoming(incoming_stream)
        .await
        .map_err(|error| BuilderError::Internal(format!("Failed to start server: {error}")))?;

    Ok(())
}
