use std::{collections::BTreeMap, sync::Arc};

use common_wit::Target;
use http::Uri;
use tokio::sync::Mutex;

use crate::{
    make_instance_id,
    protos::{
        self,
        common::ModuleId,
        runtime::{
            instantiate_module_request::ModuleReference, InstantiateModuleRequest,
            InstantiateModuleResponse,
        },
    },
    wasmtime::WasmtimeCompile,
    CommonRuntimeError, Module, ModuleInstance, ModulePreparer, PrecompiledModule, ToWasmComponent,
    WillCompileModule,
};

/// Instantiate a module using the provided [WasmtimeCompile] sandbox and cache the live instance
/// in the provided [BTreeMap] against its instance ID.
pub async fn instantiate_module(
    request: InstantiateModuleRequest,
    sandbox: Arc<Mutex<WasmtimeCompile>>,
    instances: Arc<Mutex<BTreeMap<String, Arc<Mutex<ModuleInstance>>>>>,
    builder_address: Option<Uri>,
) -> Result<InstantiateModuleResponse, CommonRuntimeError> {
    let module_reference = request.module_reference.ok_or_else(|| {
        CommonRuntimeError::ModuleInstantiationFailed("No module referenced in request".into())
    })?;

    async fn prepare_module<Mod: Module + ToWasmComponent + 'static>(
        module: Mod,
        module_id: String,
        sandbox: Arc<Mutex<WasmtimeCompile>>,
        instances: Arc<Mutex<BTreeMap<String, Arc<Mutex<ModuleInstance>>>>>,
    ) -> Result<String, CommonRuntimeError> {
        let prepared_module = {
            let mut sandbox = sandbox.lock().await;
            Arc::new(Mutex::new(ModuleInstance::WasmtimePrebuiltModule(
                sandbox.prepare(module).await?,
            )))
        };

        let mut instances = instances.lock().await;
        let instance_id = make_instance_id(&module_id)?;
        instances.insert(instance_id.clone(), prepared_module);
        Ok(instance_id) as Result<String, CommonRuntimeError>
    }

    Ok(match module_reference {
        ModuleReference::ModuleId(module_id) => {
            let module = PrecompiledModule {
                target: match module_id.target() {
                    protos::common::Target::CommonModule => Target::CommonModule,
                },
                module_id: module_id.id.clone(),
                builder_address,
            };

            let instance_id = tokio::task::spawn(prepare_module(
                module,
                module_id.id.clone(),
                sandbox,
                instances,
            ))
            .await
            .map_err(|error| {
                CommonRuntimeError::PreparationFailed(format!("Thread panic: {error}"))
            })??;

            InstantiateModuleResponse {
                module_id: Some(module_id),
                instance_id,
            }
        }
        ModuleReference::ModuleSource(module_source) => {
            let module = WillCompileModule::new(module_source.into(), builder_address);

            let (instance_id, module_id) = tokio::task::spawn(async {
                let module_id = module.id().await?.to_owned();
                let target = match module.target() {
                    Target::CommonModule => protos::common::Target::CommonModule,
                }
                .into();

                let instance_id =
                    prepare_module(module, module_id.clone(), sandbox, instances).await?;

                Ok((
                    instance_id,
                    ModuleId {
                        target,
                        id: module_id,
                    },
                )) as Result<(String, ModuleId), CommonRuntimeError>
            })
            .await
            .map_err(|error| {
                CommonRuntimeError::PreparationFailed(format!("Thread panic: {error}"))
            })??;

            InstantiateModuleResponse {
                module_id: Some(module_id),
                instance_id,
            }
        }
    })
}
