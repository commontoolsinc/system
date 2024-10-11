#![warn(missing_docs)]

//! This package implements a basic JavaScript VM for interpreting
//! `common:function`-compatible JavaScript sources. The environment supports
//! importing `common:*` APIs as the `common:module` JavaScript would otherwise
//! do in a AOT-compiled scenario. Notably: this package is designed to be
//! compiled as a Wasm Component exporting the `common:script` target. This
//! enables us to evaluate `common:module` JavaScript within a Wasm sandbox.
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
#[allow(warnings)]
#[rustfmt::skip]
mod bindings;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod module;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod types;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod util;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod guest {
    use crate::bindings::exports::common::formula::module::Guest as ModuleGuest;
    use crate::bindings::Guest as VirtualModuleGuest;
    use crate::module::Module;
    use crate::types::{Datom, Instruction, RangeQuery, Scalar, State};

    pub struct JavaScriptInterpreter;

    impl ModuleGuest for JavaScriptInterpreter {
        fn init(input: Vec<(String, Scalar)>) -> Result<(State, RangeQuery), String> {
            let module = Module::get().ok_or("No script source has been set!")?;
            let mut module = module.write().map_err(|error| format!("{error}"))?;
            module.call_init(input)
        }

        fn step(state: State, datoms: Vec<Datom>) -> Result<(State, Vec<Instruction>), String> {
            let module = Module::get().ok_or("No script source has been set!")?;
            let mut module = module.write().map_err(|error| format!("{error}"))?;
            module.call_step(state, datoms)
        }

        fn end(state: State) -> Result<Vec<Instruction>, String> {
            let module = Module::get().ok_or("No script source has been set!")?;
            let mut module = module.write().map_err(|error| format!("{error}"))?;
            module.call_end(state)
        }
    }

    impl VirtualModuleGuest for JavaScriptInterpreter {
        fn set_source(source: String) -> Result<(), String> {
            Module::load(Some(source))?;
            Ok(())
        }
    }

    impl Drop for JavaScriptInterpreter {
        fn drop(&mut self) {
            Module::reset();
        }
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
use guest::*;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
bindings::export!(JavaScriptInterpreter with_types_in bindings);
