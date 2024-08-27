use crate::{
    target::{function::NativeFunctionContext, function_vm::NativeFunctionVmContext},
    Affinity, BasicIo, CommonRuntimeError, FunctionDefinition, FunctionVmDefinition, IoShape,
    IoValues, LiveModules, ModuleBody, ModuleDefinition, ModuleDriver, ModuleFactory, ModuleId,
    ModuleManager, NativeRuntime, SourceCode,
};
use common_ifc::{Context as IfcContext, ModuleEnvironment};
use common_protos::runtime::{
    instantiate_module_request::ModuleReference, InstantiateModuleRequest,
    InstantiateModuleResponse,
};
use common_wit::Target;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Instantiate a module using the provided [WasmtimeCompile] sandbox and cache the live instance
/// in the provided [BTreeMap] against its instance ID.
pub async fn instantiate_module(
    request: InstantiateModuleRequest,
    runtime: Arc<Mutex<NativeRuntime>>,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<InstantiateModuleResponse, CommonRuntimeError> {
    let module_reference = request.module_reference.ok_or_else(|| {
        CommonRuntimeError::InvalidInstantiationParameters("No module referenced in request".into())
    })?;

    let target = match match &module_reference {
        ModuleReference::ModuleSignature(module_signature) => module_signature.target(),
        ModuleReference::ModuleSource(module_source) => module_source.target(),
    } {
        common_protos::common::Target::CommonFunction => Target::CommonFunction,
        common_protos::common::Target::CommonFunctionVm => Target::CommonFunctionVm,
    };

    let body = match module_reference {
        ModuleReference::ModuleSignature(module_signature) => {
            ModuleBody::Signature(ModuleId::Base64(module_signature.id.clone()))
        }
        ModuleReference::ModuleSource(module_source) => ModuleBody::SourceCode(
            module_source
                .source_code
                .into_iter()
                .map(|(key, value)| (key, SourceCode::from(value)))
                .collect(),
        ),
    };

    let default_input: IoValues = request.default_input.try_into()?;
    let input_shape: IoShape = IoShape::from(&default_input);
    let output_shape: IoShape = request.output_shape.try_into()?;

    let module_definition = ModuleDefinition {
        target,
        affinity: Affinity::LocalOnly,
        inputs: input_shape,
        outputs: output_shape.clone(),
        body,
    };

    let module_instance_id = match target {
        Target::CommonFunction => {
            let function_module_definition = FunctionDefinition::try_from(module_definition)?;
            let function_module_factory = runtime
                .lock()
                .await
                .prepare(function_module_definition)
                .await?;
            let function_module_instance = function_module_factory
                .instantiate(NativeFunctionContext::new(
                    BasicIo::from_initial_state(default_input, output_shape),
                    IfcContext {
                        environment: ModuleEnvironment::Server,
                    },
                ))
                .await?;
            live_modules
                .lock()
                .await
                .add(function_module_instance.into())
                .await
        }
        Target::CommonFunctionVm => {
            let function_module_definition = FunctionVmDefinition::try_from(module_definition)?;
            let function_module_factory = runtime
                .lock()
                .await
                .prepare(function_module_definition)
                .await?;
            let function_module_instance = function_module_factory
                .instantiate(NativeFunctionVmContext::new(
                    BasicIo::from_initial_state(default_input, output_shape),
                    IfcContext {
                        environment: ModuleEnvironment::Server,
                    },
                ))
                .await?;
            live_modules
                .lock()
                .await
                .add(function_module_instance.into())
                .await
        }
    };

    Ok(InstantiateModuleResponse {
        instance_id: module_instance_id.to_string(),
    })
}
