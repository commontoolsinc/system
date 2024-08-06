use crate::{CommonRuntimeError, InputOutput, ModuleInstanceId, Runtime, RuntimeIo};
use common_ifc::Policy;
use common_protos::runtime::{RunModuleRequest, RunModuleResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_module(
    request: RunModuleRequest,
    runtime: Arc<Mutex<Runtime>>,
) -> Result<RunModuleResponse, CommonRuntimeError> {
    let runtime = runtime.lock().await;
    let instance_id = ModuleInstanceId(request.instance_id);

    let output_shape = runtime.output_shape(&instance_id)?;
    let input = request.input.try_into()?;
    let policy = Policy::with_defaults()?;
    let io = RuntimeIo::new(input, output_shape.clone());
    let io = runtime.run(&instance_id, io, &policy).await?;

    let output = io.output().clone().into();

    Ok(RunModuleResponse { output })
}
