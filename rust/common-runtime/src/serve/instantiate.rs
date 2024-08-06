use crate::{CommonRuntimeError, CompiledModule, ModuleDefinition, RawModule, Runtime, RuntimeIo};
use common_protos::runtime::{
    instantiate_module_request::ModuleReference, InstantiateModuleRequest,
    InstantiateModuleResponse,
};
use common_wit::Target;
use http::Uri;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Instantiate a module using the provided [WasmtimeCompile] sandbox and cache the live instance
/// in the provided [BTreeMap] against its instance ID.
pub async fn instantiate_module(
    request: InstantiateModuleRequest,
    runtime: Arc<Mutex<Runtime>>,
    builder_address: Option<Uri>,
) -> Result<InstantiateModuleResponse, CommonRuntimeError> {
    let module_reference = request.module_reference.ok_or_else(|| {
        CommonRuntimeError::InvalidInstantiationParameters("No module referenced in request".into())
    })?;

    Ok(match module_reference {
        ModuleReference::ModuleSignature(module_signature) => {
            let module = CompiledModule {
                target: match module_signature.target() {
                    common_protos::common::Target::CommonFunction => Target::CommonFunction,
                    common_protos::common::Target::CommonFunctionVm => {
                        return Err(CommonRuntimeError::InvalidInstantiationParameters(
                            "Must provide sources to instantiate a common:function/virtual-module"
                                .into(),
                        ));
                    }
                },
                module_id: module_signature.id.clone().into(),
                builder_address,
            };

            let initial_io = RuntimeIo::from_initial_state(
                request.default_input.try_into()?,
                request.output_shape.try_into()?,
            );

            let mut runtime = runtime.lock().await;
            let instance_id = runtime.compile(module, initial_io).await?;

            InstantiateModuleResponse {
                module_signature: Some(module_signature),
                instance_id: instance_id.to_string(),
            }
        }
        ModuleReference::ModuleSource(module_source_proto) => {
            let target_proto = module_source_proto.target;
            let module_source: crate::ModuleSource = module_source_proto.into();
            let target = module_source.target;

            let module = RawModule::new(module_source, builder_address);
            let module_id = module.id().await?;

            let module_signature = common_protos::common::ModuleSignature {
                target: target_proto,
                id: module_id.to_string(),
            };

            let initial_io = RuntimeIo::from_initial_state(
                request.default_input.try_into()?,
                request.output_shape.try_into()?,
            );

            let mut runtime = runtime.lock().await;

            let instance_id = match target {
                Target::CommonFunction => runtime.compile(module, initial_io).await?,
                Target::CommonFunctionVm => runtime.interpret(module, initial_io).await?,
            };
            InstantiateModuleResponse {
                module_signature: Some(module_signature),
                instance_id: instance_id.to_string(),
            }
        }
    })
}
