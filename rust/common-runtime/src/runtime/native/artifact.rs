use std::sync::Arc;

use bytes::Bytes;
use common_protos::builder::{
    builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
    BundleSourceCodeRequest, BundleSourceCodeResponse, ReadComponentRequest, ReadComponentResponse,
};
use common_wit::Target;
use http::Uri;

use crate::{cache::Cache, module::ModuleDefinition, CommonRuntimeError, ModuleBody, ModuleId};

// NOTE: Theoretical memory ceiling is WASM_CACHE_CAPACITY * WASM_MAX_BYTE_SIZE
// + BUNDLED_SOURCE_CODE_MAX_BYTE_SIZE * BUNDLED_SOURCE_CODE_CACHE_CAPACITY

static WASM_CACHE_CAPACITY: usize = 64;
static WASM_MAX_BYTE_SIZE: usize = 32 * 1024 * 1024;

static BUNDLED_SOURCE_CODE_CACHE_CAPACITY: usize = 256;
static BUNDLED_SOURCE_CODE_MAX_BYTE_SIZE: usize = 2 * 1024 * 1024;

static JAVASCRIPT_COMMON_FUNCTION_INTERPRETER: Bytes = Bytes::from_static(include_bytes!(env!(
    "JAVASCRIPT_COMMON_FUNCTION_INTERPRETER_WASM_PATH"
)));

/// Well-known virtual module interpreters that may be requested from an [ArtifactResolver]
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum VirtualModuleInterpreter {
    /// A JavaScript interpreter that emulates a `common:function/module`
    JavaScriptFunction,
}

/// An [ArtifactResolver] is a one-stop shop for accessing
#[derive(Clone)]
pub struct ArtifactResolver {
    builder_address: Option<Uri>,
    wasm_cache: Cache<ModuleId, Bytes>,
    bundled_source_code_cache: Cache<ModuleId, Arc<String>>,
}

impl ArtifactResolver {
    /// Instantiate a new [ArtifactResolver], optionally providing a [Uri] that
    /// refers to a Builder gRPC server
    pub fn new(builder_address: Option<Uri>) -> Result<Self, CommonRuntimeError> {
        Ok(Self {
            builder_address,
            wasm_cache: Cache::new(WASM_CACHE_CAPACITY)?,
            bundled_source_code_cache: Cache::new(BUNDLED_SOURCE_CODE_CACHE_CAPACITY)?,
        })
    }

    /// Look up the Wasm artifact for a well-known [VirtualModuleInterpreter]
    pub async fn get_virtual_module_interpreter_wasm(
        &self,
        kind: VirtualModuleInterpreter,
    ) -> Result<Bytes, CommonRuntimeError> {
        match kind {
            VirtualModuleInterpreter::JavaScriptFunction => {
                Ok(JAVASCRIPT_COMMON_FUNCTION_INTERPRETER.clone())
            }
        }
    }

    /// Given a [ModuleDefinition], resolve the Wasm artifact represents the
    /// substantive implementation of the corresponding Module.
    pub async fn get_module_wasm(
        &self,
        definition: &ModuleDefinition,
    ) -> Result<Bytes, CommonRuntimeError> {
        let id = ModuleId::from(definition);

        if let Some(item) = self.wasm_cache.get(&id).await {
            Ok(item)
        } else if let Some(address) = &self.builder_address {
            let mut builder_client = BuilderClient::connect(address.to_string())
                .await?
                .max_encoding_message_size(WASM_MAX_BYTE_SIZE)
                .max_decoding_message_size(WASM_MAX_BYTE_SIZE);

            // TODO: Align IDs as derived by builder with IDs as derived by runtime
            let id = if let ModuleBody::SourceCode(source_code_collection) = &definition.body {
                let BuildComponentResponse { id } = builder_client
                    .build_component(tonic::Request::new(BuildComponentRequest {
                        module_source: Some(common_protos::common::ModuleSource {
                            target: common_protos::common::Target::CommonFunction.into(),
                            source_code: source_code_collection
                                .iter()
                                .map(|(key, value)| (key.to_owned(), value.into()))
                                .collect(),
                        }),
                    }))
                    .await
                    .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
                    .into_inner();
                ModuleId::Base64(id)
            } else {
                id
            };

            let ReadComponentResponse { component } = builder_client
                .read_component(tonic::Request::new(ReadComponentRequest {
                    id: id.to_string(),
                }))
                .await
                .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
                .into_inner();

            let bytes = Bytes::from(component);

            self.wasm_cache.insert(id, bytes.clone()).await;

            Ok(bytes)
        } else {
            Err(CommonRuntimeError::PreparationFailed(
                "Wasm component bytes not cached, and no builder address was configured"
                    .to_string(),
            ))
        }
    }

    /// Given a [ModuleDefinition] with a [ModuleBody::SourceCode] body, resolve a bundled
    /// artifact that combines all of its source code inputs
    pub async fn get_bundled_source_code(
        &self,
        definition: &ModuleDefinition,
    ) -> Result<Arc<String>, CommonRuntimeError> {
        let id = ModuleId::from(definition);

        if let Some(item) = self.bundled_source_code_cache.get(&id).await {
            Ok(item)
        } else if let Some(address) = &self.builder_address {
            let mut builder_client = BuilderClient::connect(address.to_string())
                .await?
                .max_encoding_message_size(BUNDLED_SOURCE_CODE_MAX_BYTE_SIZE)
                .max_decoding_message_size(BUNDLED_SOURCE_CODE_MAX_BYTE_SIZE);

            let target = match definition.target {
                Target::CommonFunction | Target::CommonFunctionVm => {
                    common_protos::common::Target::CommonFunction
                }
            };

            let source_code: std::collections::HashMap<String, common_protos::common::SourceCode> =
                match &definition.body {
                    crate::ModuleBody::Signature(_) => {
                        return Err(CommonRuntimeError::PreparationFailed(
                            "Cannot bundle source code when module body is 'signature'".into(),
                        ));
                    }
                    crate::ModuleBody::SourceCode(source_code_collection) => source_code_collection
                        .iter()
                        .map(|(name, source_code)| (name.to_owned(), source_code.into()))
                        .collect(),
                };

            let BundleSourceCodeResponse {
                bundled_source_code,
            } = builder_client
                .bundle_source_code(tonic::Request::new(BundleSourceCodeRequest {
                    module_source: Some(common_protos::common::ModuleSource {
                        target: target.into(),
                        source_code,
                    }),
                }))
                .await
                .map_err(|error| CommonRuntimeError::PreparationFailed(format!("{error}")))?
                .into_inner();

            let bundled_source_code = Arc::new(bundled_source_code);

            self.bundled_source_code_cache
                .insert(id, bundled_source_code.clone())
                .await;

            Ok(bundled_source_code)
        } else {
            Err(CommonRuntimeError::PreparationFailed(
                "Source code bundle not cached, and no builder address was configured".to_string(),
            ))
        }
    }
}
