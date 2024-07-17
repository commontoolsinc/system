#![warn(missing_docs)]

//! This package implements a basic JavaScript VM for interpreting
//! `common:module`-compatible JavaScript sources. The environment supports
//! importing `common:*` APIs as the `common:module` JavaScript would otherwise
//! do in a AOT-compiled scenario. Notably: this package is designed to be
//! compiled as a Wasm Component exporting the `common:script` target. This
//! enables us to evaluate `common:module` JavaScript within a Wasm sandbox.

#[allow(warnings)]
mod bindings;

mod data;
mod io;
mod module;

use bindings::Guest;
// use io::read_script;
use module::Module;

struct JavaScriptInterpreter;

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

bindings::export!(JavaScriptInterpreter with_types_in bindings);
