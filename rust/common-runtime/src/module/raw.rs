use super::{ModuleDefinition, ToModuleSources, ToWasmComponent};
// REASON: They will be used when we fill out the wasm32 impls
#[cfg_attr(target_arch = "wasm32", allow(unused_imports))]
use crate::{
    CommonRuntimeError, ContentType, ModuleId, ModuleSource, SourceCode,
    COMMON_JAVASCRIPT_INTERPRETER_WASM,
};
use async_trait::async_trait;
use bytes::Bytes;

#[cfg(not(target_arch = "wasm32"))]
use common_protos::{
    builder::{
        builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
        ReadComponentRequest, ReadComponentResponse,
    },
    MAX_MESSAGE_SIZE,
};
use common_wit::Target;
use http::Uri;
use std::collections::BTreeMap;
use tokio::sync::OnceCell;

/// A [RawModule] embodies all the source information necessary to
/// compile a Common Module as a Wasm Component.
// REASON: Dead code will not be dead when we fill out wasm32 impls
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
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

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    async fn wasm(&self) -> Result<(ModuleId, Bytes), CommonRuntimeError> {
        todo!()
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn wasm(&self) -> Result<(ModuleId, Bytes), CommonRuntimeError> {
        match self.module_source.target {
            Target::CommonModule => {
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
                                "Needed to build module, but not builder address was configured"
                                    .into(),
                            ));
                        };

                        let BuildComponentResponse { id } = client
                            .build_component(BuildComponentRequest {
                                module_source: Some(self.module_source.clone().into()),
                            })
                            .await
                            .map_err(|error| {
                                CommonRuntimeError::PreparationFailed(format!("{error}"))
                            })?
                            .into_inner();

                        let ReadComponentResponse { component } = client
                            .read_component(tonic::Request::new(ReadComponentRequest { id }))
                            .await
                            .map_err(|error| {
                                CommonRuntimeError::PreparationFailed(format!("{error}"))
                            })?
                            .into_inner();

                        let id = ModuleId::Hash(blake3::hash(&component));
                        Ok((id, component.into()))
                    })
                    .await?;
                Ok((id.clone(), bytes.clone()))
            }
            Target::CommonScript => {
                let (_, entrypoint) = self.module_source.entrypoint()?;
                match entrypoint.content_type {
                    ContentType::JavaScript => {
                        let id = ModuleId::Hash(blake3::hash(COMMON_JAVASCRIPT_INTERPRETER_WASM));
                        Ok((id, COMMON_JAVASCRIPT_INTERPRETER_WASM.into()))
                    }
                    ContentType::Python => Err(CommonRuntimeError::InvalidInstantiationParameters(
                        "Instantiating python as a common:script is not supported yet".into(),
                    )),
                }
            }
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ModuleDefinition for RawModule {
    fn target(&self) -> Target {
        self.module_source.target
    }

    async fn id(&self) -> Result<ModuleId, CommonRuntimeError> {
        let (id, _) = self.wasm().await?;
        Ok(id)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ToModuleSources for RawModule {
    async fn to_module_sources(
        &self,
    ) -> Result<Option<BTreeMap<String, SourceCode>>, CommonRuntimeError> {
        Ok(Some(self.module_source.source_code.clone()))
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ToWasmComponent for RawModule {
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError> {
        let (_, bytes) = self.wasm().await?;
        Ok(bytes)
    }
}
