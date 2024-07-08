use boa_engine::{
    class::{Class, ClassBuilder},
    object::builtins::JsUint8Array,
    property::PropertyDescriptor,
    Context, JsArgs, JsData, JsError, JsObject, JsResult, JsString, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};

use crate::bindings::common::{
    data::types::Reference as HostReference, io::state::Value as HostValue,
};

#[repr(transparent)]
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Reference {
    #[unsafe_ignore_trace]
    pub inner: HostReference,
}

impl Reference {
    pub fn read(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(reference) = object.downcast_ref::<Reference>() {
                let name = args
                    .get_or_undefined(0)
                    .as_string()
                    .ok_or_else(|| {
                        JsError::from_opaque(JsValue::String("No key specified".into()))
                    })?
                    .to_std_string()
                    .map_err(|error| {
                        JsError::from_opaque(JsValue::String(format!("{error}").into()))
                    })?;

                let Some(reference) = reference.inner.read(&name) else {
                    return Ok(JsValue::Undefined);
                };

                return Ok(JsValue::Object(Reference::from_data(
                    Reference { inner: reference },
                    context,
                )?));
            }
        }

        Err(JsError::from_opaque(JsValue::String(
            format!("Unexpected context").into(),
        )))
    }

    pub fn deref(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(reference) = object.downcast_ref::<Reference>() {
                let Some(value) = reference.inner.deref().map_err(|error| {
                    JsError::from_opaque(JsValue::String(format!("{error}").into()))
                })?
                else {
                    return Ok(JsValue::Undefined);
                };

                let (tag, val) = match value {
                    HostValue::String(string) => ("string", JsValue::String(string.into())),
                    HostValue::Number(number) => ("number", JsValue::Rational(number)),
                    HostValue::Boolean(boolean) => ("boolean", JsValue::Boolean(boolean)),
                    HostValue::Buffer(buffer) => (
                        "buffer",
                        JsValue::Object(
                            JsUint8Array::from_iter(buffer.into_iter(), context)?.into(),
                        ),
                    ),
                };

                let object = JsObject::default();
                object.insert_property(
                    JsString::from("tag"),
                    PropertyDescriptor::builder()
                        .value(JsValue::String(tag.into()))
                        .build(),
                );
                object.insert_property(
                    JsString::from("val"),
                    PropertyDescriptor::builder().value(val).build(),
                );

                return Ok(JsValue::Object(object));
            }
        }

        Err(JsError::from_opaque(JsValue::String(
            format!("Unexpected context").into(),
        )))
    }
}

impl Class for Reference {
    const NAME: &'static str = "Reference";

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            JsString::from("deref"),
            0,
            NativeFunction::from_fn_ptr(Self::deref),
        );

        class.method(
            JsString::from("read"),
            0,
            NativeFunction::from_fn_ptr(Self::read),
        );

        Ok(())
    }

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Err(JsError::from_opaque(JsValue::String(
            "Illegal constructor".into(),
        )))
    }
}
