use crate::{CommonRuntimeError, ModuleDefinition, ModuleId, ToWasmComponent};
use async_trait::async_trait;
use bytes::Bytes;
use common_protos::{
    builder::{builder_client::BuilderClient, ReadComponentRequest, ReadComponentResponse},
    MAX_MESSAGE_SIZE,
};
use common_wit::Target;
use http::Uri;

/// A [CompiledModule] represents a reference to a Common Module that is already
/// compiled as a fully-fledged Wasm Component. The compiled artifact is
/// referenced by the `module_id` field.
#[derive(Debug, Clone)]
pub struct CompiledModule {
    /// The Common Module Target expected to be implemented by the referenced
    /// artifact
    pub target: Target,
    /// The unique identifier that references the compiled artifact
    pub module_id: ModuleId,
    /// An optional address of a `common-builder` server, which will be used to
    /// fetch the compiled artifact if configured
    pub builder_address: Option<Uri>,
}

#[async_trait]
impl ModuleDefinition for CompiledModule {
    fn target(&self) -> Target {
        self.target
    }

    async fn id(&self) -> Result<ModuleId, CommonRuntimeError> {
        Ok(self.module_id.clone())
    }
}

#[async_trait]
impl ToWasmComponent for CompiledModule {
    async fn to_wasm_component(&self) -> Result<Bytes, CommonRuntimeError> {
        let mut client = if let Some(address) = &self.builder_address {
            BuilderClient::connect(address.to_string())
                .await?
                .max_encoding_message_size(MAX_MESSAGE_SIZE)
                .max_decoding_message_size(MAX_MESSAGE_SIZE)
        } else {
            return Err(CommonRuntimeError::PreparationFailed(
                "Needed to build module, but no builder address was configured".into(),
            ));
        };

        info!("Reading component");
        let ReadComponentResponse { component } = client
            .read_component(tonic::Request::new(ReadComponentRequest {
                id: self.module_id.to_string(),
            }))
            .await
            .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
            .into_inner();

        Ok(component.into())
    }
}
