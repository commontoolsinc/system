use crate::{bindings::common::basic::host_callback::callback as host_callback, util};
use blake3::Hash;
use boa_engine::module::{ModuleLoader, Referrer};
use boa_engine::property::Attribute;
use boa_engine::{
    builtins::promise::PromiseState, js_string, Context, JsObject, Module as JsModule,
    NativeFunction, Source,
};
use boa_engine::{JsResult, JsString, JsValue};
use boa_runtime::Console;
use once_cell::unsync::OnceCell;
use std::collections::BTreeMap;
use std::{cell::RefCell, thread_local};
use std::{rc::Rc, sync::RwLock};

thread_local! {
    pub static MODULE_STATE: RefCell<OnceCell<Rc<RwLock<Module>>>> = const { RefCell::new(OnceCell::new()) };
}

#[derive(Default)]
pub struct CommonModuleLoader {
    builtins: RefCell<BTreeMap<JsString, JsModule>>,
}

/*
impl CommonModuleLoader {
    pub fn new() -> Self {
        Self {
            builtins: RefCell::new(BTreeMap::new()),
        }
    }
}
*/

impl ModuleLoader for CommonModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: Referrer,
        _specifier: JsString,
        _finish_load: Box<dyn FnOnce(JsResult<JsModule>, &mut Context)>,
        _context: &mut Context,
    ) {
        /*
        println!("Loading imported module: {:?}", specifier);
        let maybe_module = self.builtins.borrow().get(&specifier).cloned();
        let module = match maybe_module {
            None => {
                let specifier_std = specifier.to_std_string().unwrap_or_default();
                let module: boa_engine::Module = match specifier_std.as_str() {
                    //"common:io/state@0.0.1" => create_io_state_module(context),
                    _ => {
                        finish_load(
                            Err(JsError::from_opaque(js_string!("Invalid specifier").into())),
                            context,
                        );
                        return;
                    }
                };

                self.builtins
                    .borrow_mut()
                    .insert(specifier.clone(), module.clone());
                module
            }
            Some(module) => module,
        };
        finish_load(Ok(module), context);
        */
    }

    fn register_module(&self, _specifier: boa_engine::JsString, _module: JsModule) {}

    fn get_module(&self, specifier: JsString) -> Option<JsModule> {
        self.builtins.borrow().get(&specifier).cloned()
    }

    fn init_import_meta(
        &self,
        _import_meta: &JsObject,
        _module: &JsModule,
        _context: &mut Context,
    ) {
        // TODO(#35): If we don't configure this correctly, we may inadvertently expose that
        // the environment is a VM running within Wasm.
    }
}

pub struct Module {
    run_fn: JsObject,
    context: Context,
    script_id: Rc<Hash>,
}

impl Module {
    pub fn call_run(&mut self, input: String) -> Result<String, String> {
        let init = self.run_fn.clone();
        let input_js = util::str_to_js_object(input, &mut self.context)?;
        let result = init
            .call(&JsValue::undefined(), &[input_js], &mut self.context)
            .map_err(util::format_error)?;
        util::js_object_to_str(result, &mut self.context)
    }

    pub fn load(maybe_script: Option<String>) -> Result<Rc<RwLock<Module>>, String> {
        let script_id = Rc::new(blake3::hash(
            maybe_script.clone().unwrap_or_default().as_bytes(),
        ));
        let maybe_script = maybe_script.map(Rc::new);

        let state = Self::get_or_init(maybe_script.clone(), script_id.clone())?;
        let read_state = state.read().map_err(|error| format!("{error}"))?;

        if read_state.script_id == script_id {
            Ok(state.clone())
        } else {
            Self::reset();
            Self::get_or_init(maybe_script, script_id)
        }
    }

    pub fn get() -> Option<Rc<RwLock<Module>>> {
        MODULE_STATE.with_borrow(|state| state.get().cloned())
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

                        let loader = Rc::new(CommonModuleLoader::default());

                        let mut context = Context::builder()
                            .module_loader(loader.clone())
                            .build()
                            .map_err(|error| format!("{error}"))?;

                        let console = Console::init(&mut context);
                        context
                            .register_global_property(
                                js_string!(Console::NAME),
                                console,
                                Attribute::all(),
                            )
                            .map_err(|error| format!("{error}"))?;

                        let source = Source::from_bytes(script.as_bytes());
                        let module = JsModule::parse(source, None, &mut context)
                            .map_err(|error| format!("{error}"))?;

                        let module_evaluates = module.load_link_evaluate(&mut context);

                        context.run_jobs();

                        match module_evaluates.state() {
                            PromiseState::Fulfilled(_) => {
                                //println!("Module evaluated")
                            }
                            PromiseState::Pending => {
                                //println!("Module didn't evaluate!")
                            }
                            PromiseState::Rejected(error) => {
                                //println!("Module error: {}", error.display())
                            }
                        };

                        set_host_callback(&mut context)?;

                        let run_fn = get_module_export("run", &module, &mut context)?;

                        Ok(Rc::new(RwLock::new(Module {
                            run_fn,
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

fn get_module_export(
    export_name: &str,
    module: &JsModule,
    context: &mut Context,
) -> Result<JsObject, String> {
    module
        .namespace(context)
        .get(js_string!(export_name), context)
        .map_err(|error| format!("Failed to get '{export_name}' export: {error}"))?
        .as_callable()
        .cloned()
        .ok_or_else(|| format!("No '{export_name}' function was exported!"))
}

fn set_host_callback(context: &mut Context) -> Result<(), String> {
    context
        .register_global_builtin_callable(
            js_string!("hostCallback"),
            1,
            NativeFunction::from_fn_ptr(|_, args, context| {
                let Some(arg) = args.get(0) else {
                    return Err(util::str_to_js_error(
                        "`hostCallback` requires at least one argument.",
                    ));
                };
                let payload =
                    util::js_object_to_str(arg.clone(), context).map_err(util::str_to_js_error)?;
                let response = host_callback(&payload).map_err(util::str_to_js_error)?;
                Ok(util::str_to_js_object(response, context).map_err(util::str_to_js_error)?)
            }),
        )
        .map_err(|e| e.to_string())
}
