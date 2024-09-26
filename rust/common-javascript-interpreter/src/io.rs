use crate::{
    bindings::common::io::state::{self, Value as HostValue},
    data::Reference,
    util::js_error,
};
use boa_engine::{
    class::Class, js_string, module::SyntheticModuleInitializer, object::FunctionObjectBuilder,
    Context, JsArgs, JsError, JsResult, JsValue, Module, NativeFunction,
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

            let value = js_to_host_value(context, args.get_or_undefined(1))?;

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

fn js_to_host_value(context: &mut Context, obj: &JsValue) -> JsResult<HostValue> {
    const VAL_TYPE_MISMATCH: &str = "'val' type does not match 'tag' type";

    let JsValue::Object(object) = obj else {
        return Err(js_error("Write received an unsupported type"));
    };

    let tag = object
        .get(js_string!("tag"), context)?
        .as_string()
        .ok_or_else(|| js_error("Unexpected type for 'tag' property"))?
        .to_std_string()
        .map_err(|error| js_error(format!("{error}")))?;

    let val = object.get(js_string!("val"), context)?;

    match tag.as_str() {
        "string" => {
            let value = val
                .as_string()
                .ok_or_else(|| js_error(VAL_TYPE_MISMATCH))?
                .to_std_string()
                .map_err(|error| js_error(format!("{error}")))?;
            Ok(HostValue::String(value))
        }
        "number" => {
            let value = val.as_number().ok_or_else(|| js_error(VAL_TYPE_MISMATCH))?;
            Ok(HostValue::Number(value))
        }
        "boolean" => {
            let value = val
                .as_boolean()
                .ok_or_else(|| js_error(VAL_TYPE_MISMATCH))?;
            Ok(HostValue::Boolean(value))
        }
        "buffer" => {
            todo!();
        }
        t => Err(js_error(format!("Unknown 'tag' type '{t}'."))),
    }
}
