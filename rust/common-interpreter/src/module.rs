use crate::io::create_io_state_module;
use blake3::Hash;
use boa_engine::property::Attribute;
use boa_engine::JsValue;
use boa_engine::{
    builtins::promise::PromiseState, js_string, Context, JsObject, Module as JsModule, Source,
};
use boa_runtime::Console;
use once_cell::unsync::OnceCell;
use std::{cell::RefCell, thread_local};
use std::{rc::Rc, sync::RwLock};

thread_local! {
    pub static MODULE_STATE: RefCell<OnceCell<Rc<RwLock<Module>>>> = RefCell::new(OnceCell::new());
}

pub struct Module {
    run: JsObject,
    context: Context,
    script_id: Rc<Hash>,
}

impl Module {
    pub fn run(&mut self) -> Result<(), String> {
        let run = self.run.clone();
        run.call(&JsValue::undefined(), &[], &mut self.context)
            .map_err(|error| format!("{error}"))?;
        Ok(())
    }

    pub fn load(maybe_script: Option<String>) -> Result<Rc<RwLock<Module>>, String> {
        let script_id = Rc::new(blake3::hash(
            maybe_script.clone().unwrap_or_default().as_bytes(),
        ));
        let maybe_script = maybe_script.map(|script| Rc::new(script));

        let state = Self::get_or_init(maybe_script.clone(), script_id.clone())?;
        let read_state = state.read().map_err(|error| format!("{error}"))?;

        if read_state.script_id == script_id {
            Ok(state.clone())
        } else {
            Self::reset();
            Self::get_or_init(maybe_script, script_id)
        }
    }

    pub fn reset() {
        MODULE_STATE.with_borrow_mut(|state| {
            state.take();
        });
    }

    fn get_or_init(
        maybe_script: Option<Rc<String>>,
        script_id: Rc<Hash>,
    ) -> Result<Rc<RwLock<Module>>, String> {
        Ok(MODULE_STATE
            .try_with(move |state| {
                let state = state.borrow_mut();
                let result = state
                    .get_or_try_init(move || {
                        let script = if let Some(script) = maybe_script {
                            script
                        } else {
                            return Err("Must provide a script to load!".to_owned());
                        };

                        let mut context = Context::default();

                        let console = Console::init(&mut context);
                        context
                            .register_global_property(
                                js_string!(Console::NAME),
                                console,
                                Attribute::all(),
                            )
                            .map_err(|error| format!("{error}"))?;

                        context.module_loader().register_module(
                            js_string!("common:io@0.0.1/state"),
                            create_io_state_module(&mut context),
                        );

                        let source = Source::from_bytes(script.as_bytes());
                        let module = JsModule::parse(source, None, &mut context)
                            .map_err(|error| format!("{error}"))?;

                        let module_evaluates = module.load_link_evaluate(&mut context);

                        context.run_jobs();

                        match module_evaluates.state() {
                            PromiseState::Fulfilled(_) => println!("Module evaluated"),
                            PromiseState::Pending => println!("Module didn't evaluate!"),
                            PromiseState::Rejected(error) => {
                                println!("Module error: {}", error.display())
                            }
                        };

                        let run = module
                            .namespace(&mut context)
                            .get(js_string!("run"), &mut context)
                            .map_err(|error| format!("{error}"))?
                            .as_callable()
                            .cloned()
                            .ok_or_else(|| "No 'run' function was exported!".to_owned())?;

                        Ok(Rc::new(RwLock::new(Module {
                            run,
                            context,
                            script_id,
                        }))) as Result<_, String>
                    })?
                    .clone();
                Ok(result) as Result<Rc<RwLock<Module>>, String>
            })
            .map_err(|error| format!("{error}"))??
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Module;

    #[test]
    fn it_runs_a_common_module() -> Result<(), String> {
        let script = r#"export const run = () => console.log('hello');"#.to_owned();

        let module = Module::load(Some(script))?;
        let mut module = module.write().map_err(|error| format!("{error}"))?;

        for _ in 0..3 {
            module.run()?;
        }

        Module::reset();

        Ok(())
    }
}
