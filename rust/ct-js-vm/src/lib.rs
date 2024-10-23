#![warn(missing_docs)]

//! This crate implements a basic JavaScript VM with
//! minimal API during development.
//!
//! Building for wasm32-wasip1 results in an empty lib.

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
    use crate::bindings::exports::common::basic::processor::Guest as ProcessorGuest;
    use crate::bindings::exports::common::basic::vm::Guest as VmGuest;
    use crate::module::Module;

    pub struct JavaScriptInterpreter;

    impl ProcessorGuest for JavaScriptInterpreter {
        fn run(input: String) -> Result<String, String> {
            let module = Module::get().ok_or("No script source has been set!")?;
            let mut module = module.write().map_err(|error| format!("{error}"))?;
            module.call_run(input)
        }
    }

    impl VmGuest for JavaScriptInterpreter {
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
