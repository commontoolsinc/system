#![warn(missing_docs)]

//! This package implements a basic JavaScript VM for interpreting
//! `common:module`-compatible JavaScript sources. The environment supports
//! importing `common:*` APIs as the `common:module` JavaScript would otherwise
//! do in a AOT-compiled scenario. Notably: this package is designed to be
//! compiled as a Wasm Component exporting the `common:script` target. This
//! enables us to evaluate `common:module` JavaScript within a Wasm sandbox.
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
#[allow(warnings)]
mod bindings;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod data;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod io;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod module;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod guest {
    use crate::bindings::Guest;
    // use io::read_script;
    use crate::module::Module;

    pub struct JavaScriptInterpreter;

    impl Guest for JavaScriptInterpreter {
        fn run() -> Result<(), String> {
            let module = Module::get().ok_or("No script source has been set!")?;
            let mut module = module.write().map_err(|error| format!("{error}"))?;

            module.run()?;

            Ok(())
        }

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
