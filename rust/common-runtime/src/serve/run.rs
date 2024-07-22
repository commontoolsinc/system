use crate::{CommonRuntimeError, ModuleInstanceId, Runtime, RuntimeIo};
use common_protos::{
    self as protos,
    runtime::{RunModuleRequest, RunModuleResponse},
};
use common_wit::{InputOutput, Value};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::Mutex;

pub async fn run_module(
    request: RunModuleRequest,
    runtime: Arc<Mutex<Runtime>>,
) -> Result<RunModuleResponse, CommonRuntimeError> {
    let runtime = runtime.lock().await;
    let instance_id = ModuleInstanceId(request.instance_id);

    let output_shape = runtime.output_shape(&instance_id)?;
    let mut input = BTreeMap::new();
    for (key, value) in request.input.into_iter() {
        input.insert(
            key,
            Value::try_from(value).map_err(|_| CommonRuntimeError::InvalidValue)?,
        );
    }
    let io = RuntimeIo::new(input, output_shape.clone());
    let io = runtime.run(&instance_id, io).await?;

    let output = io
        .output()
        .clone()
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect::<HashMap<String, protos::common::Value>>();

    Ok(RunModuleResponse { output })
}
