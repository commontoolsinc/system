//! Uses [wasm_component_layer] to implement a backend
//! for both `wasmtime` and a browser's WASM implementation.

use crate::backends::context::Context;
use crate::sync::{ConditionalSend, ConditionalSync};
use crate::{
    Engine, Error, HostFeatures, Instance, Module, ModuleDefinition, Result, VirtualMachine,
};
use std::collections::HashMap;
use wasm_component_layer as wcl;

#[cfg(target_arch = "wasm32")]
use js_wasm_runtime_layer::Engine as InnerEngine;
#[cfg(not(target_arch = "wasm32"))]
use wasmtime_runtime_layer::Engine as InnerEngine;

type Store = wcl::Store<Context, InnerEngine>;

/// An implementation of [`Engine`] via [`wasm_component_layer`],
/// primarily for `wasm32-unknown-unknown` via `js_wasm_runtime_layer`.
pub struct WclEngine<H: HostFeatures> {
    engine: wcl::Engine<InnerEngine>,
    vm_components: HashMap<VirtualMachine, wcl::Component>,
    host_features: std::marker::PhantomData<H>,
}

impl<H> WclEngine<H>
where
    H: HostFeatures,
{
    /// Create a new [`WclEngine`].
    pub fn new(vms: Vec<VirtualMachine>) -> Result<Self> {
        let engine = wcl::Engine::new(InnerEngine::default());
        let mut vm_components = HashMap::default();
        for vm in vms {
            let component = wcl::Component::new(&engine, vm.as_bytes())
                .map_err(|e| Error::from(e.to_string()))?;
            vm_components.insert(vm, component);
        }

        Ok(WclEngine {
            engine,
            vm_components,
            host_features: Default::default(),
        })
    }
}

impl<H> Engine for WclEngine<H>
where
    H: HostFeatures,
{
    type Module = WclModule<H>;

    fn module(&self, definition: ModuleDefinition) -> Result<Self::Module> {
        let component = self
            .vm_components
            .get(&definition.vm)
            .ok_or(Error::UnsupportedVm)?
            .to_owned();
        let linker = wcl::Linker::default();

        Ok(WclModule::<H> {
            linker,
            engine: self.engine.clone(),
            component,
            definition,
            host_features: Default::default(),
        })
    }
}

/// An implementation of [`Module`] via [`wasm_component_layer`].
pub struct WclModule<H: HostFeatures> {
    linker: wcl::Linker,
    engine: wcl::Engine<InnerEngine>,
    component: wcl::Component,
    definition: ModuleDefinition,
    host_features: std::marker::PhantomData<H>,
}

impl<H> Module for WclModule<H>
where
    H: HostFeatures,
{
    type Instance = WclInstance;
    fn instantiate(&mut self) -> Result<Self::Instance> {
        let context = Context::default();
        let mut store = Store::new(&self.engine, context);

        Interface::Identifier("wasi:random/random@0.2.0")
            .set_fn::<(u64,), (Vec<u8>,), _>(
                &mut store,
                &mut self.linker,
                "get-random-bytes",
                |mut ctx, params| {
                    let store: &mut Context = ctx.data_mut();
                    Ok((store.get_random_bytes(params.0),))
                },
            )
            .map_err(|e| Error::LinkerFailure(e.to_string()))?;

        Interface::Identifier("common:basic/host-callback@0.0.1")
            .set_fn::<(String,), (std::result::Result<String, String>,), _>(
                &mut store,
                &mut self.linker,
                "callback",
                |_ctx, params| Ok((H::host_callback(params.0),)),
            )
            .map_err(|e| Error::LinkerFailure(e.to_string()))?;

        let module_instance = self
            .linker
            .instantiate(&mut store, &self.component)
            .map_err(|e| Error::InstantiationFailure(e.to_string()))?;

        WclInstance::new(store, module_instance, &self.definition)
    }
}

/// An implementation of [`Instance`] via [`wasm_component_layer`].
pub struct WclInstance {
    run_fn: wcl::TypedFunc<String, std::result::Result<String, String>>,
    module_instance: wcl::Instance,
    store: Store,
}

impl WclInstance {
    fn new(
        mut store: Store,
        module_instance: wcl::Instance,
        definition: &ModuleDefinition,
    ) -> Result<Self> {
        let set_source_fn = Interface::Identifier("common:basic/vm@0.0.1")
            .get_fn::<String, std::result::Result<(), String>>(&module_instance, "set-source")?;
        let _ = set_source_fn
            .call(&mut store, definition.source.clone())
            .map_err(|e| Error::InstantiationFailure(e.to_string()))?;

        let run_fn = Interface::Identifier("common:basic/processor@0.0.1")
            .get_fn::<String, std::result::Result<String, String>>(&module_instance, "run")?;

        Ok(Self {
            module_instance,
            store,
            run_fn,
        })
    }
}

impl Instance for WclInstance {
    fn run(&mut self, input: String) -> Result<String> {
        self.run_fn
            .call(&mut self.store, input)
            .map_err(|e| Error::InvocationFailure(e.to_string()))?
            .map_err(|e| Error::InvocationFailure(e.to_string()))
    }
}

enum Interface {
    Root,
    Identifier(&'static str),
}

impl Interface {
    fn get_fn<I: wcl::ComponentList, O: wcl::ComponentList>(
        &self,
        instance: &wcl::Instance,
        func_name: &str,
    ) -> Result<wcl::TypedFunc<I, O>> {
        let export_instance = match self {
            Interface::Root => instance.exports().root(),
            Interface::Identifier(identifier) => {
                let interface_id = wcl::InterfaceIdentifier::try_from(*identifier)
                    .map_err(|e| Error::InstantiationFailure(e.to_string()))?;
                instance
                    .exports()
                    .instance(&interface_id)
                    .ok_or_else(|| Error::InstantiationFailure("No interface found.".into()))?
            }
        };
        let func = export_instance
            .func(func_name)
            .ok_or_else(|| {
                Error::InstantiationFailure(format!("No '{}' function found.", func_name))
            })?
            .typed::<I, O>()
            .map_err(|e| Error::InstantiationFailure(e.to_string()))?;
        Ok(func)
    }

    fn set_fn<I: wcl::ComponentList, O: wcl::ComponentList, F>(
        &self,
        store: &mut Store,
        linker: &mut wcl::Linker,
        func_name: &str,
        f: F,
    ) -> Result<()>
    where
        F: 'static
            + ConditionalSync
            + Fn(
                wcl::StoreContextMut<'_, Context, InnerEngine>,
                I,
            ) -> std::result::Result<O, anyhow::Error>
            + ConditionalSend,
    {
        let linker_instance = match self {
            Interface::Root => linker.root_mut(),
            Interface::Identifier(identifier) => {
                let interface_id = wcl::InterfaceIdentifier::try_from(*identifier)
                    .map_err(|e| Error::LinkerFailure(e.to_string()))?;
                if let Some(instance) = linker.instance_mut(&interface_id) {
                    instance
                } else {
                    linker
                        .define_instance(interface_id)
                        .map_err(|e| Error::LinkerFailure(e.to_string()))?
                }
            }
        };
        let typed_func = wcl::TypedFunc::new(store, f);
        linker_instance
            .define_func(func_name, typed_func.func())
            .map_err(|e| Error::LinkerFailure(e.to_string()))?;
        Ok(())
    }
}
