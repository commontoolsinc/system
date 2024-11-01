use crate::{
    artifact::Artifact,
    error::Error,
    storage::{JsComponentStorage, PersistedHashStorage},
    JavaScriptBundler,
};
use async_trait::async_trait;
use blake3::Hash;
use ct_common::{ContentType, ModuleDefinition};
use ct_protos::builder::{
    builder_server::Builder as BuilderProto, BuildComponentRequest, BuildComponentResponse,
    ReadComponentRequest, ReadComponentResponse,
};
use std::str::FromStr;
use tonic::{Request, Response, Status};

pub struct BuildComponentConfig {
    definition: ModuleDefinition,
    bundle_common_imports: bool,
}

#[derive(Clone)]
pub struct Builder {
    storage: PersistedHashStorage,
}

impl Builder {
    pub fn new(storage: PersistedHashStorage) -> Self {
        Self { storage }
    }

    pub async fn build(&self, config: BuildComponentConfig) -> Result<Hash, Error> {
        info!("Building source: {:#?}", config.definition.source);
        let artifact = match config.definition.content_type {
            ContentType::JavaScript => {
                JavaScriptBundler::bundle_from_bytes_sync(config.definition.source.into()).await?
            }
        };
        self.storage.write(artifact).await
    }

    pub async fn read(&self, hash: Hash) -> Result<Artifact, Error> {
        self.storage.read(&hash).await?.ok_or(Error::ModuleNotFound)
    }
}

#[async_trait]
impl BuilderProto for Builder {
    async fn build_component(
        &self,
        request: Request<BuildComponentRequest>,
    ) -> Result<Response<BuildComponentResponse>, Status> {
        let request = request.into_inner();
        let definition: ModuleDefinition = request
            .module_definition
            .ok_or(Error::BadRequest)?
            .try_into()
            .map_err(|_| Error::BadRequest)?;

        let id = self
            .build(BuildComponentConfig {
                definition,
                bundle_common_imports: false,
            })
            .await?;

        Ok(Response::new(BuildComponentResponse {
            component_id: id.to_string(),
        }))
    }

    async fn read_component(
        &self,
        request: Request<ReadComponentRequest>,
    ) -> Result<Response<ReadComponentResponse>, Status> {
        let request = request.into_inner();
        let hash = Hash::from_str(&request.component_id)
            .map_err(|error| Status::invalid_argument(format!("Could not parse ID: {error}")))?;
        let artifact = self.read(hash).await?;
        Ok(Response::new(ReadComponentResponse::from(artifact)))
    }
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::BadRequest => Status::invalid_argument("Request was malformed"),
            Error::InvalidConfiguration(error) => Status::failed_precondition(error),
            Error::InvalidModule(error) => Status::invalid_argument(error),
            Error::ModuleNotFound => Status::internal("Module not found"),
            Error::Internal(error) => Status::internal(error),
        }
    }
}
