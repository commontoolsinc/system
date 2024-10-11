use crate::{
    BasicIo, CommonRuntimeError, Function, FunctionInterface, HasModuleContext, InputOutput,
    LiveModules, ModuleContext, ModuleInstanceId, ModuleManager, Validated,
};
use common_ifc::Policy;
use common_protos::runtime::{RunModuleRequest, RunModuleResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_module(
    request: RunModuleRequest,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<RunModuleResponse, CommonRuntimeError> {
    let instance_id = ModuleInstanceId(request.instance_id);
    let live_modules = live_modules.lock().await;
    let function = match request.keep_alive {
        true => live_modules
            .get(&instance_id)
            .await
            .ok_or(CommonRuntimeError::UnknownInstanceId(instance_id))?,
        false => live_modules
            .take(&instance_id)
            .await
            .ok_or(CommonRuntimeError::UnknownInstanceId(instance_id))?,
    };
    let input = request.input.try_into()?;
    let policy = Policy::with_defaults()?;

    let output = match function {
        Function::Module(function) => {
            let mut function = function.lock().await;

            let output_shape = function.context().io().output_shape().clone();
            let io = BasicIo::new(input, output_shape);
            let validated_io = Validated::try_from((policy, function.context().ifc(), io))?;

            function.run(validated_io).await?
        }
        Function::VirtualModule(function) => {
            let mut function = function.lock().await;

            let output_shape = function.context().io().output_shape().clone();
            let io = BasicIo::new(input, output_shape);
            let validated_io = Validated::try_from((policy, function.context().ifc(), io))?;

            function.run(validated_io).await?
        }
        _ => {
            return Err(CommonRuntimeError::InternalError(
                "Unexpected function type.".into(),
            ))
        }
    };

    Ok(RunModuleResponse {
        output: output.into(),
    })
}
