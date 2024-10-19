use crate::backends::context::Context;
use crate::{Engine, Error, Instance, Module, ModuleDefinition, Result, VirtualMachine};
use async_trait::async_trait;
use std::collections::HashMap;
use wasmtime::{
    component::{Component, Linker},
    AsContextMut,
};

mod virtual_module {
    wasmtime::component::bindgen!({
        world: "virtual-module",
        path: "../../wit/common/basic/wit",
        async: true
    });
}

use virtual_module::VirtualModule;

/// An implementation of [`Engine`] via [`wasmtime`].
pub struct WasmtimeEngine {
    engine: wasmtime::Engine,
    vm_components: HashMap<VirtualMachine, Component>,
}

impl WasmtimeEngine {
    /// Create a new [`WasmtimeEngine`].
    pub fn new(vms: Vec<VirtualMachine>) -> Result<Self> {
        let engine = {
            let mut config = wasmtime::Config::default();

            config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            config.async_support(true);
            config.wasm_backtrace(true);

            wasmtime::Engine::new(&config).map_err(|e| Error::from(e.to_string()))
        }?;
        let mut vm_components = HashMap::default();
        for vm in vms {
            let component =
                Component::new(&engine, vm.as_bytes()).map_err(|e| Error::from(e.to_string()))?;
            vm_components.insert(vm, component);
        }
        Ok(WasmtimeEngine {
            engine,
            vm_components,
        })
    }
}

#[async_trait]
impl Engine for WasmtimeEngine {
    type Module = WasmtimeModule;

    async fn module(&self, definition: ModuleDefinition) -> Result<Self::Module> {
        let component = self
            .vm_components
            .get(&definition.vm)
            .ok_or(Error::UnsupportedVm)?
            .to_owned();
        let mut linker = Linker::new(&self.engine);

        let mut random_interface = linker
            .instance("wasi:random/random@0.2.0")
            .map_err(|e| Error::LinkerFailure(e.to_string()))?;
        random_interface
            .func_wrap::<_, (u64,), (Vec<u8>,)>("get-random-bytes", |mut ctx, params| {
                let store: &mut Context = ctx.data_mut();
                Ok((store.get_random_bytes(params.0),))
            })
            .map_err(|e| Error::LinkerFailure(e.to_string()))?;

        Ok(WasmtimeModule {
            linker,
            engine: self.engine.clone(),
            component,
            definition,
        })
    }
}

/// An implementation of [`Module`] via [`wasmtime`].
pub struct WasmtimeModule {
    linker: Linker<Context>,
    engine: wasmtime::Engine,
    component: Component,
    definition: ModuleDefinition,
}

#[async_trait]
impl Module for WasmtimeModule {
    type Instance = WasmtimeInstance;
    async fn instantiate(&mut self) -> Result<Self::Instance> {
        let context = Context::default();
        let mut store = wasmtime::Store::new(&self.engine, context);

        let module_instance =
            VirtualModule::instantiate_async(&mut store, &self.component, &self.linker)
                .await
                .map_err(|e| Error::InstantiationFailure(e.to_string()))?;

        module_instance
            .common_basic_vm()
            .call_set_source(&mut store, &self.definition.source)
            .await
            .map_err(|e| Error::InstantiationFailure(e.to_string()))?
            .map_err(|e| Error::InstantiationFailure(e.to_string()))?;

        Ok(WasmtimeInstance {
            module_instance,
            store,
        })
    }
}

/// An implementation of [`Instance`] via [`wasmtime`].
pub struct WasmtimeInstance {
    module_instance: VirtualModule,
    store: wasmtime::Store<Context>,
}

#[async_trait]
impl Instance for WasmtimeInstance {
    async fn run(&mut self, input: String) -> Result<String> {
        let value = self
            .module_instance
            .common_basic_processor()
            .call_run(self.store.as_context_mut(), &input)
            .await
            .map_err(|e| Error::InvocationFailure(e.to_string()))?
            .map_err(|e| Error::InvocationFailure(e.to_string()))?;
        Ok(value)
    }
}
