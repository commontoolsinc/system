use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    target::{
        formula_vm::NativeFormulaVm, function::NativeFunction, function_vm::NativeFunctionVm,
    },
    Module, ModuleInstanceId, ModuleManager,
};

/// A set of variants for well-known implementors of [crate::FunctionInterface]
#[derive(Clone)]
pub enum Function {
    /// The `common:function/module` variant of a [crate::FunctionInterface]
    Module(Arc<Mutex<NativeFunction>>),
    /// The `common:function/virtual-module` variant of a [crate::FunctionInterface]
    VirtualModule(Arc<Mutex<NativeFunctionVm>>),
    /// The `common:formula/virtual-module` variant of a [crate::FunctionInterface]
    VirtualFormula(Arc<Mutex<NativeFormulaVm>>),
}

impl Function {
    async fn instance_id(&self) -> ModuleInstanceId {
        match self {
            Function::Module(module) => module.lock().await.instance_id().clone(),
            Function::VirtualModule(module) => module.lock().await.instance_id().clone(),
            Function::VirtualFormula(module) => module.lock().await.instance_id().clone(),
        }
    }
}

impl From<NativeFunction> for Function {
    fn from(value: NativeFunction) -> Self {
        Function::Module(Arc::new(Mutex::new(value)))
    }
}

impl From<NativeFunctionVm> for Function {
    fn from(value: NativeFunctionVm) -> Self {
        Function::VirtualModule(Arc::new(Mutex::new(value)))
    }
}

impl From<NativeFormulaVm> for Function {
    fn from(value: NativeFormulaVm) -> Self {
        Function::VirtualFormula(Arc::new(Mutex::new(value)))
    }
}

/// A type that retains references to live instances of various kinds of
/// modules, intended to be used within a long-running process such as a
/// web server
#[derive(Default)]
pub struct LiveModules {
    functions: Arc<Mutex<BTreeMap<ModuleInstanceId, Function>>>,
}

#[async_trait]
impl ModuleManager<Function> for LiveModules {
    async fn add(&self, module: Function) -> ModuleInstanceId {
        let instance_id = module.instance_id().await;
        self.functions
            .lock()
            .await
            .insert(instance_id.clone(), module);
        instance_id
    }

    async fn get(&self, id: &ModuleInstanceId) -> Option<Function> {
        self.functions.lock().await.get(id).cloned()
    }

    async fn take(&self, id: &ModuleInstanceId) -> Option<Function> {
        self.functions.lock().await.remove(id)
    }
}
