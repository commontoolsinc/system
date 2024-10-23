use crate::util::format_error;
use boa_engine::{js_string, Context, JsValue};

const INVALID_JS_TYPE: &str = "Could not cast JS value.";

pub fn str_to_js_object(value: String, context: &mut Context) -> Result<JsValue, String> {
    json_parse(JsValue::String(js_string!(value)), context)
}

pub fn js_object_to_str(value: JsValue, context: &mut Context) -> Result<String, String> {
    let parsed = json_stringify(value, context)?
        .as_string()
        .ok_or(INVALID_JS_TYPE)?
        .to_std_string_escaped();
    Ok(parsed.into())
}

fn json_parse(data: JsValue, context: &mut Context) -> Result<JsValue, String> {
    let json = context.intrinsics().objects().json();
    let parse = json
        .get(js_string!("parse"), context)
        .map_err(format_error)?;
    parse
        .as_callable()
        .ok_or(INVALID_JS_TYPE)?
        .call(&JsValue::from(json), &[data], context)
        .map_err(format_error)
}

fn json_stringify(data: JsValue, context: &mut Context) -> Result<JsValue, String> {
    let json = context.intrinsics().objects().json();
    let stringify = json
        .get(js_string!("stringify"), context)
        .map_err(format_error)?;
    stringify
        .as_callable()
        .ok_or(INVALID_JS_TYPE)?
        .call(&JsValue::from(json), &[data], context)
        .map_err(format_error)
}
