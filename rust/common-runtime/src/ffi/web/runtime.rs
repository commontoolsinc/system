use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use common_ifc::ModuleEnvironment;
use http::Uri;
use wasm_bindgen::prelude::*;

use crate::{
    ffi::{
        web::{cast::to_string, host::JavaScriptModuleDefinition},
        CommonFunction, ModuleDefinition,
    },
    remote::function::WebRemoteFunctionContext,
    BasicIo, ModuleDriver, ModuleFactory, ModuleInstanceId, ModuleManager, WebRuntime,
};
use async_trait::async_trait;

use super::FunctionVariant;

/// The [`CommonRuntime`] constitutes the JavaScript-facing bindings into
/// the Common Runtime.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CommonRuntime {
    inner: Rc<RefCell<WebRuntime>>,
    functions: Rc<RefCell<BTreeMap<ModuleInstanceId, FunctionVariant>>>,
}

#[wasm_bindgen]
impl CommonRuntime {
    /// Construct a new [`CommonRuntime`], passing an optional URL to a backing
    /// remote Runtime that will be used when instantiating and invoking remote
    /// modules
    #[wasm_bindgen(constructor)]
    pub fn new(remote_runtime_address: Option<String>) -> Self {
        let remote_runtime_address = if let Some(raw_address) = remote_runtime_address {
            Some(
                Uri::try_from(raw_address)
                    .map_err(|error| format!("Failed to parse runtime address: {error}"))
                    .unwrap(),
            )
        } else {
            None
        };

        CommonRuntime {
            inner: Rc::new(RefCell::new(
                WebRuntime::new(remote_runtime_address)
                    .map_err(|error| format!("Failed to construct Common Runtime: {error}"))
                    .unwrap(),
            )),
            functions: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    /// Instantiate a module given some module definition
    pub async fn instantiate(
        &mut self,
        definition: JavaScriptModuleDefinition,
    ) -> Result<CommonFunction, String> {
        let (module_definition, default_inputs) =
            ModuleDefinition::interpret_host_definition(definition)?;

        match module_definition {
            ModuleDefinition::RemoteFunction(remote_function_definition) => {
                let io = BasicIo::from_initial_state(
                    default_inputs,
                    remote_function_definition.outputs.clone(),
                );
                let factory = self
                    .inner
                    .borrow()
                    .prepare(remote_function_definition)
                    .await
                    .map_err(to_string)?;
                let context = WebRemoteFunctionContext::new(
                    io,
                    common_ifc::Context {
                        environment: ModuleEnvironment::Server,
                    },
                );

                let instance = FunctionVariant::RemoteModule(Rc::new(RefCell::new(
                    factory.instantiate(context).await.map_err(to_string)?,
                )));

                self.add(instance.clone()).await;

                Ok(CommonFunction::from(instance).into())
            }
        }
    }
}

#[async_trait(?Send)]
impl ModuleManager<FunctionVariant> for CommonRuntime {
    async fn add(&self, module_instance: FunctionVariant) -> ModuleInstanceId {
        let instance_id = module_instance.instance_id();
        self.functions
            .borrow_mut()
            .insert(instance_id.clone(), module_instance);
        instance_id
    }
    async fn get(&self, id: &ModuleInstanceId) -> Option<FunctionVariant> {
        self.functions.borrow().get(id).cloned()
    }
    async fn take(&self, id: &ModuleInstanceId) -> Option<FunctionVariant> {
        self.functions.borrow_mut().remove(id)
    }
}
