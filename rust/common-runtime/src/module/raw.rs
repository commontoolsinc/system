use async_trait::async_trait;
use bytes::Bytes;
use http::Uri;
use std::collections::BTreeMap;
use tokio::sync::OnceCell;

use common_wit::Target;

use crate::{
    protos::{
        builder::{
            builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
            ReadComponentRequest, ReadComponentResponse,
        },
        MAX_MESSAGE_SIZE,
    },
    CommonRuntimeError, ModuleId, ModuleSource, SourceCode,
};

use super::{ModuleDefinition, ToModuleSources, ToWasmComponent};

/// A [RawModule] embodies all the source information necessary to
/// compile a Common Module as a Wasm Component.
#[derive(Debug, Clone)]
pub struct RawModule {
    module_source: ModuleSource,
    builder_address: Option<Uri>,
    wasm: OnceCell<(ModuleId, Bytes)>,
}

impl RawModule {
    /// Instantiate the [RawModule]. It will only be able to compile if
    /// a `builder_address` is provided.
    pub fn new(module_source: ModuleSource, builder_address: Option<Uri>) -> Self {
        Self {
            module_source,
            builder_address,
            wasm: OnceCell::new(),
        }
    }

    async fn wasm(&self) -> Result<(&ModuleId, Bytes), CommonRuntimeError> {
        let (id, bytes) = self
            .wasm
            .get_or_try_init(|| async {
                let mut client = if let Some(address) = &self.builder_address {
                    BuilderClient::connect(address.to_string())
                        .await?
                        .max_decoding_message_size(MAX_MESSAGE_SIZE)
                        .max_encoding_message_size(MAX_MESSAGE_SIZE)
                } else {
                    return Err(CommonRuntimeError::PreparationFailed(
                        "Needed to build module, but not builder address was configured".into(),
                    ));
                };

                let BuildComponentResponse { id } = client
                    .build_component(BuildComponentRequest {
                        module_source: Some(self.module_source.clone().into()),
                    })
                    .await
                    .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
                    .into_inner();

                let ReadComponentResponse { component } = client
                    .read_component(tonic::Request::new(ReadComponentRequest { id }))
                    .await
                    .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
                    .into_inner();

                let id = ModuleId::Hash(blake3::hash(&component));
                Ok((id, component.into()))
            })
            .await?;
        Ok((id, bytes.clone()))
    }
}

#[async_trait]
impl ModuleDefinition for RawModule {
    fn target(&self) -> Target {
        self.module_source.target
    }

    async fn id(&self) -> Result<&ModuleId, CommonRuntimeError> {
        let (id, _) = self.wasm().await?;
        Ok(id)
    }
}

#[async_trait]
impl ToModuleSources for RawModule {
    async fn to_module_sources(
        &self,
    ) -> Result<Option<BTreeMap<String, SourceCode>>, CommonRuntimeError> {
        Ok(Some(self.module_source.source_code.clone()))
    }
}

#[async_trait]
impl ToWasmComponent for RawModule {
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError> {
        let (_, bytes) = self.wasm().await?;
        Ok(bytes)
    }
}
