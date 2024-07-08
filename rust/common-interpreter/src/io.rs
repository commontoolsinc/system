use crate::{
    bindings::common::io::state::{self, Value as HostValue},
    data::Reference,
};
use boa_engine::{
    class::Class, js_string, module::SyntheticModuleInitializer, object::FunctionObjectBuilder,
    Context, JsArgs, JsError, JsValue, Module, NativeFunction,
};

pub fn read_script() -> Option<String> {
    match state::read(&String::from("script")) {
        Some(reference) => match reference.deref() {
            Ok(Some(HostValue::String(script))) => Some(script),
            _ => None,
        },
        _ => None,
    }
}

pub fn create_io_state_module(context: &mut Context) -> Module {
    let read = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, context| {
            let name = args
                .get_or_undefined(0)
                .as_string()
                .ok_or_else(|| JsError::from_opaque(JsValue::String("No key specified".into())))?
                .to_std_string()
                .map_err(|error| {
                    JsError::from_opaque(JsValue::String(format!("{error}").into()))
                })?;

            let maybe_reference = if let Some(inputs) = state::read(&String::from("input")) {
                inputs.read(&name)
            } else {
                None
            };

            let Some(reference) = maybe_reference else {
                return Ok(JsValue::undefined());
            };

            Ok(JsValue::Object(Reference::from_data(
                Reference { inner: reference },
                context,
            )?))
        }),
    )
    .build();

    let write = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, _context| {
            let name = args
                .get_or_undefined(0)
                .as_string()
                .ok_or_else(|| JsError::from_opaque(JsValue::String("No key specified".into())))?
                .to_std_string()
                .map_err(|error| {
                    JsError::from_opaque(JsValue::String(format!("{error}").into()))
                })?;

            let value = match args.get_or_undefined(1) {
                JsValue::Null | JsValue::Undefined | JsValue::BigInt(_) | JsValue::Symbol(_) => {
                    todo!()
                }
                JsValue::Boolean(boolean) => HostValue::Boolean(*boolean),
                JsValue::String(string) => {
                    HostValue::String(string.to_std_string().unwrap_or_default())
                }
                JsValue::Rational(number) => HostValue::Number(*number),
                JsValue::Integer(number) => HostValue::Number(*number as f64),
                JsValue::Object(_object) => todo!("Uint8Array support"),
            };

            state::write(&name, &value);

            Ok(JsValue::undefined())
        }),
    )
    .build();

    Module::synthetic(
        &[js_string!("read")],
        SyntheticModuleInitializer::from_copy_closure_with_captures(
            |module, fns, _| {
                module.set_export(&js_string!("read"), fns.0.clone().into())?;
                module.set_export(&js_string!("write"), fns.1.clone().into())?;
                Ok(())
            },
            (read, write),
        ),
        None,
        context,
    )
}
