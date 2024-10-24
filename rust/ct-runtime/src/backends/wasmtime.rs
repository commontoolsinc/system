use crate::backends::{context::Context, EngineBackend, InstanceBackend, ModuleBackend};
use crate::{Error, HostCallback, HostCallbackFn, ModuleDefinition, Result, VirtualMachine};
use std::collections::HashMap;
use wasmtime::{
    component::{Component, Linker},
    AsContextMut,
};

mod virtual_module {
    wasmtime::component::bindgen!({
        world: "virtual-module",
        path: "../../wit/common/basic/wit",
    });
}

/// An implementation of [`Engine`] via [`wasmtime`].
pub struct WasmtimeEngine {
    engine: wasmtime::Engine,
    vm_components: HashMap<VirtualMachine, Component>,
    callback: HostCallback,
}

impl WasmtimeEngine {
    /// Create a new [`WasmtimeEngine`].
    pub fn new(callback: impl HostCallbackFn, vms: Vec<VirtualMachine>) -> Result<Self> {
        let engine = {
            let mut config = wasmtime::Config::default();

            config.cranelift_opt_level(wasmtime::OptLevel::Speed);
            config.wasm_backtrace(true);
            //config.async_support(true);

            wasmtime::Engine::new(&config).map_err(|e| Error::from(e.to_string()))
        }?;
        let mut vm_components = HashMap::default();
        for vm in vms {
            let component =
                Component::new(&engine, vm.as_bytes()).map_err(|e| Error::from(e.to_string()))?;
            vm_components.insert(vm, component);
        }
        let callback = HostCallback::new(callback);
        Ok(WasmtimeEngine {
            engine,
            vm_components,
            callback,
        })
    }
}

impl EngineBackend for WasmtimeEngine {
    type Module = WasmtimeModule;

    fn module(&self, definition: ModuleDefinition) -> Result<Self::Module> {
        let component = self
            .vm_components
            .get(&definition.vm)
            .ok_or(Error::UnsupportedVm)?
            .to_owned();
        let linker = create_linker(self.callback.clone(), &self.engine)?;
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

impl ModuleBackend for WasmtimeModule {
    type Instance = WasmtimeInstance;
    fn instantiate(&mut self) -> Result<Self::Instance> {
        let context = Context::new();
        let mut store = wasmtime::Store::new(&self.engine, context);

        let module_instance =
            virtual_module::VirtualModule::instantiate(&mut store, &self.component, &self.linker)
                .map_err(|e| Error::InstantiationFailure(e.to_string()))?;

        module_instance
            .common_basic_vm()
            .call_set_source(&mut store, &self.definition.source)
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
    module_instance: virtual_module::VirtualModule,
    store: wasmtime::Store<Context>,
}

impl InstanceBackend for WasmtimeInstance {
    fn run(&mut self, input: String) -> Result<String> {
        let value = self
            .module_instance
            .common_basic_processor()
            .call_run(self.store.as_context_mut(), &input)
            .map_err(|e| Error::InvocationFailure(e.to_string()))?
            .map_err(|e| Error::InvocationFailure(e.to_string()))?;
        Ok(value)
    }
}

fn create_linker(
    host_callback: HostCallback,
    engine: &wasmtime::Engine,
) -> Result<Linker<Context>> {
    let mut linker = Linker::new(engine);

    let mut random_interface = linker
        .instance("wasi:random/random@0.2.0")
        .map_err(|e| Error::LinkerFailure(e.to_string()))?;
    random_interface
        .func_wrap::<_, (u64,), (Vec<u8>,)>("get-random-bytes", |mut ctx, params| {
            let store: &mut Context = ctx.data_mut();
            Ok((store.get_random_bytes(params.0),))
        })
        .map_err(|e| Error::LinkerFailure(e.to_string()))?;

    let mut callback_interface = linker
        .instance("common:basic/host-callback@0.0.1")
        .map_err(|e| Error::LinkerFailure(e.to_string()))?;
    callback_interface
        .func_wrap::<_, (String,), (std::result::Result<String, String>,)>(
            "callback",
            move |_ctx, params| Ok((host_callback.invoke(params.0),)),
        )
        .map_err(|e| Error::LinkerFailure(e.to_string()))?;
    Ok(linker)
}
