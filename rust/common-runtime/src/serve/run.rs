use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::{
    protos::{
        self,
        runtime::{RunModuleRequest, RunModuleResponse},
    },
    CommonRuntimeError, InputOutput, ModuleInstance, PreparedModule, Value,
};

#[derive(Debug)]
pub struct HandlerIo {
    input: HashMap<String, protos::common::Value>,
    // TODO: This needs to be constrained at instantiation time (see runtime.proto)
    output: BTreeMap<String, Value>,
}

impl HandlerIo {
    pub fn new(input: HashMap<String, protos::common::Value>) -> Self {
        Self {
            input,
            output: BTreeMap::new(),
        }
    }
}

impl InputOutput for HandlerIo {
    fn read(&self, key: &str) -> Option<Value> {
        if let Some(value) = self.input.get(key) {
            Value::try_from(value.clone()).ok()
        } else {
            None
        }
    }

    fn write(&mut self, key: &str, value: Value) {
        self.output.insert(key.into(), value);
    }

    fn output(&self) -> &BTreeMap<String, Value> {
        &self.output
    }
}

pub async fn run_module(
    request: RunModuleRequest,
    instances: Arc<Mutex<BTreeMap<String, Arc<Mutex<ModuleInstance>>>>>,
) -> Result<RunModuleResponse, CommonRuntimeError> {
    let instance_id = request.instance_id;
    let input = request.input;
    let instances = instances.lock().await;
    let instance = instances.get(&instance_id).ok_or_else(|| {
        CommonRuntimeError::ModuleRunFailed(format!("No module instance found for '{instance_id}'"))
    })?;

    let io = Box::new(HandlerIo::new(input));

    let instance = instance.lock().await;
    let io = instance.call(io).await?;

    let output = io
        .output()
        .clone()
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect::<HashMap<String, protos::common::Value>>();

    Ok(RunModuleResponse { output })
}
