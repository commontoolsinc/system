use crate::{
    target::formula_vm::{Datom, NativeFormulaVmContext, ScalarMap},
    Affinity, CommonRuntimeError, FormulaVmDefinition, LiveModules, ModuleDefinition, ModuleDriver,
    ModuleFactory, ModuleInstanceId, ModuleManager, NativeRuntime,
};
use common_protos::formula::{
    InstantiateFormulaRequest, InstantiateFormulaResponse, RunEndFormulaRequest,
    RunEndFormulaResponse, RunInitFormulaRequest, RunInitFormulaResponse, RunStepFormulaRequest,
    RunStepFormulaResponse,
};
use common_wit::Target;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::Function;

/// Instantiate a module using the provided[`WasmtimeCompile`] sandbox and cache the live instance
/// in the provided[`BTreeMap`] against its instance ID.
pub async fn instantiate_formula(
    mut request: InstantiateFormulaRequest,
    runtime: Arc<Mutex<NativeRuntime>>,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<InstantiateFormulaResponse, CommonRuntimeError> {
    let target = request.target().into();
    let module_reference = request.module_reference.take().ok_or_else(|| {
        CommonRuntimeError::InvalidInstantiationParameters("No module referenced in request".into())
    })?;
    let body = module_reference.try_into()?;

    let module_definition = ModuleDefinition {
        target,
        affinity: Affinity::LocalOnly,
        inputs: Default::default(),
        outputs: Default::default(),
        body,
    };

    let module_instance_id = match target {
        Target::CommonFormulaVm => {
            let function_module_definition = FormulaVmDefinition::try_from(module_definition)?;
            let function_module_factory = runtime
                .lock()
                .await
                .prepare(function_module_definition)
                .await?;
            let function_module_instance = function_module_factory
                .instantiate(NativeFormulaVmContext::default())
                .await?;
            live_modules
                .lock()
                .await
                .add(function_module_instance.into())
                .await
        }
        _ => {
            return Err(CommonRuntimeError::InvalidInstantiationParameters(
                "Unsupported target.".into(),
            ))
        }
    };

    Ok(InstantiateFormulaResponse {
        instance_id: module_instance_id.to_string(),
    })
}

pub async fn run_init_formula(
    request: RunInitFormulaRequest,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<RunInitFormulaResponse, CommonRuntimeError> {
    let instance_id = ModuleInstanceId(request.instance_id);
    let live_modules = live_modules.lock().await;
    let function = live_modules
        .get(&instance_id)
        .await
        .ok_or(CommonRuntimeError::UnknownInstanceId(instance_id))?;
    let input = ScalarMap::try_from(request.input)?;

    let (state, range_query) = match function {
        Function::VirtualFormula(function) => {
            let mut function = function.lock().await;
            function.init(&input).await?
        }
        _ => {
            return Err(CommonRuntimeError::InternalError(
                "Unexpected function type.".into(),
            ))
        }
    };

    Ok(RunInitFormulaResponse {
        state,
        range_query: Some(range_query.into()),
    })
}

pub async fn run_step_formula(
    request: RunStepFormulaRequest,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<RunStepFormulaResponse, CommonRuntimeError> {
    let instance_id = ModuleInstanceId(request.instance_id);
    let live_modules = live_modules.lock().await;
    let function = live_modules
        .get(&instance_id)
        .await
        .ok_or(CommonRuntimeError::UnknownInstanceId(instance_id))?;
    let state = &request.state;
    let datoms = request
        .datoms
        .into_iter()
        .map(Datom::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let (state, instructions) = match function {
        Function::VirtualFormula(function) => {
            let mut function = function.lock().await;
            function.step(state, datoms).await?
        }
        _ => {
            return Err(CommonRuntimeError::InternalError(
                "Unexpected function type.".into(),
            ))
        }
    };

    Ok(RunStepFormulaResponse {
        state,
        instructions: instructions.into_iter().map(|i| i.into()).collect(),
    })
}

pub async fn run_end_formula(
    request: RunEndFormulaRequest,
    live_modules: Arc<Mutex<LiveModules>>,
) -> Result<RunEndFormulaResponse, CommonRuntimeError> {
    let instance_id = ModuleInstanceId(request.instance_id);
    let live_modules = live_modules.lock().await;
    let function = live_modules
        .get(&instance_id)
        .await
        .ok_or(CommonRuntimeError::UnknownInstanceId(instance_id))?;
    let state = &request.state;
    let instructions = match function {
        Function::VirtualFormula(function) => {
            let mut function = function.lock().await;
            function.end(state).await?
        }
        _ => {
            return Err(CommonRuntimeError::InternalError(
                "Unexpected function type.".into(),
            ))
        }
    };

    Ok(RunEndFormulaResponse {
        instructions: instructions.into_iter().map(|i| i.into()).collect(),
    })
}
