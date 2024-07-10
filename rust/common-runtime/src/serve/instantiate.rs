use std::sync::Arc;

use common_wit::Target;
use http::Uri;
use tokio::sync::Mutex;

use crate::{
    protos::{
        self,
        runtime::{
            instantiate_module_request::ModuleReference, InstantiateModuleRequest,
            InstantiateModuleResponse,
        },
    },
    CommonRuntimeError, CompiledModule, ModuleDefinition, RawModule, Runtime, RuntimeIo,
};

/// Instantiate a module using the provided [WasmtimeCompile] sandbox and cache the live instance
/// in the provided [BTreeMap] against its instance ID.
pub async fn instantiate_module(
    request: InstantiateModuleRequest,
    runtime: Arc<Mutex<Runtime>>,
    builder_address: Option<Uri>,
) -> Result<InstantiateModuleResponse, CommonRuntimeError> {
    let module_reference = request.module_reference.ok_or_else(|| {
        CommonRuntimeError::ModuleInstantiationFailed("No module referenced in request".into())
    })?;

    Ok(match module_reference {
        ModuleReference::ModuleSignature(module_signature) => {
            let module = CompiledModule {
                target: match module_signature.target() {
                    protos::common::Target::CommonModule => Target::CommonModule,
                },
                module_id: module_signature.id.clone().into(),
                builder_address,
            };

            let initial_io = RuntimeIo::try_from((request.default_input, request.output_shape))?;

            let mut runtime = runtime.lock().await;
            let instance_id = runtime.compile(module, initial_io).await?;

            InstantiateModuleResponse {
                module_signature: Some(module_signature),
                instance_id: instance_id.to_string(),
            }
        }
        ModuleReference::ModuleSource(module_source) => {
            let target = module_source.target;
            let module = RawModule::new(module_source.into(), builder_address);
            let module_id = module.id().await?.clone();
            let module_signature = protos::common::ModuleSignature {
                target,
                id: module_id.to_string(),
            };

            let initial_io = RuntimeIo::try_from((request.default_input, request.output_shape))?;

            let mut runtime = runtime.lock().await;
            let instance_id = runtime.compile(module, initial_io).await?;

            InstantiateModuleResponse {
                module_signature: Some(module_signature),
                instance_id: instance_id.to_string(),
            }
        }
    })
}
