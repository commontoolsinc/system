use crate::{
    bindings::common::io::state::{self, Value as HostValue},
    data::Reference,
};
use boa_engine::{
    class::Class, js_string, module::SyntheticModuleInitializer, object::FunctionObjectBuilder,
    Context, JsArgs, JsError, JsValue, Module, NativeFunction,
};

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

            let Some(reference) = state::read(&name) else {
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
        NativeFunction::from_fn_ptr(|_, args, context| {
            let name = args
                .get_or_undefined(0)
                .as_string()
                .ok_or_else(|| JsError::from_opaque(JsValue::String("No key specified".into())))?
                .to_std_string()
                .map_err(|error| {
                    JsError::from_opaque(JsValue::String(format!("{error}").into()))
                })?;

            let value = match args.get_or_undefined(1) {
                JsValue::Object(object) => {
                    let tag = object
                        .get(js_string!("tag"), context)?
                        .as_string()
                        .ok_or_else(|| {
                            JsError::from_opaque(JsValue::String(js_string!(
                                "Unexpected type for 'tag' property"
                            )))
                        })?
                        .to_std_string()
                        .map_err(|error| {
                            JsError::from_opaque(JsValue::String(format!("{error}").into()))
                        })?;

                    let val = object.get(js_string!("val"), context)?;

                    match tag.as_str() {
                        "string" => {
                            let value = val
                                .as_string()
                                .ok_or_else(|| {
                                    JsError::from_opaque(JsValue::String(
                                        "Unexpected type for 'tag' property".into(),
                                    ))
                                })?
                                .to_std_string()
                                .map_err(|error| {
                                    JsError::from_opaque(JsValue::String(format!("{error}").into()))
                                })?;
                            Ok(HostValue::String(value))
                        }
                        _ => todo!("FUUU"),
                    }
                }
                _ => Err(JsError::from_opaque(JsValue::String(
                    "Write received an unsupported type".into(),
                ))),
            }?;

            state::write(&name, &value);

            Ok(JsValue::undefined())
        }),
    )
    .build();

    Module::synthetic(
        &[js_string!("read"), js_string!("write")],
        SyntheticModuleInitializer::from_copy_closure_with_captures(
            |module, fns, context| {
                context.register_global_class::<Reference>()?;
                module.set_export(&js_string!("read"), fns.0.clone().into())?;
                module.set_export(&js_string!("write"), fns.1.clone().into())?;
                Ok(())
            },
            (read, write),
        ),
        None,
        None,
        context,
    )
}
