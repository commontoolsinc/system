#[allow(warnings)]
mod bindings;

mod data;
mod io;
mod module;

use bindings::Guest;
use io::read_script;
use module::Module;

struct JavaScriptInterpreter;

impl Guest for JavaScriptInterpreter {
    fn run() -> Result<(), String> {
        let maybe_script = read_script();

        let module = Module::load(maybe_script)?;
        let mut module = module.write().map_err(|error| format!("{error}"))?;

        module.run()?;

        Ok(())
    }
}

impl Drop for JavaScriptInterpreter {
    fn drop(&mut self) {
        Module::reset();
    }
}

bindings::export!(JavaScriptInterpreter with_types_in bindings);
