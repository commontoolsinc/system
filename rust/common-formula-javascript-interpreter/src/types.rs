//! While the WIT types are generated in Rust via `bindings.rs`,
//! we re-export here and provide additional implementations
//! and conversions to/from their JS representations.

pub use crate::bindings::exports::common::formula::module::{
    AttributeRangeQuery, Datom, Entity, EntityRangeQuery, Instruction, RangeQuery, Scalar, State,
    ValueRangeQuery,
};
use crate::{bindings::exports::common::formula::module::Fact, util::format_error};
use boa_engine::{
    builtins::array_buffer::ArrayBuffer,
    js_string,
    object::builtins::{JsArray, JsArrayBuffer, JsUint8Array},
    property::PropertyDescriptor,
    Context, JsObject, JsValue,
};

const INVALID_JS_TYPE: &str = "Could not convert from JS value.";

const OP_ASSERT: &str = "Assert";
const OP_RETRACT: &str = "Retract";
const OP_IMPORT: &str = "Import";
const RANGE_QUERY_BY_ENTITY: &str = "ByEntity";
const RANGE_QUERY_BY_ATTRIBUTE: &str = "ByAttribute";
const RANGE_QUERY_BY_VALUE: &str = "ByValue";

macro_rules! js_object {
    ($context:expr, { $($key:expr => $value:expr),+ }) => (
        {
            let out = JsObject::with_null_proto();
            let mut err: Option<String> = None;
        $(
            let maybe_js_value = $value.into_js($context);
            if err.is_none() && maybe_js_value.is_ok() {
                let js_value = maybe_js_value.unwrap();
                if let Err(e) = out.create_data_property(js_string!($key), js_value, $context).map_err(|e| format!("{}", e)) {
                    err = Some(e);
                }
            } else if let Err(e) = maybe_js_value {
                err = Some(e);
            }
        )+
            if let Some(e) = err {
                Err(e)
            } else {
                Ok(JsValue::Object(out))
            }
        }
    )
}

/// The [`IntoJs`] trait maps the Rust types generated
/// by wit definitions via `bindings.rs` into JavaScript
/// values used by the Boa runtime.
pub trait IntoJs {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String>;
}

/// The [`FromJs`] trait maps [`JsValue`] types back
/// into their Rust/wit bindings.
pub trait FromJs
where
    Self: Sized,
{
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String>;
}

impl<T> FromJs for Option<T>
where
    T: FromJs,
{
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        if value.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(T::from_js(value, context)?))
        }
    }
}

impl FromJs for String {
    fn from_js(value: JsValue, _: &mut Context) -> Result<Self, String> {
        value
            .as_string()
            .map(|s| s.to_std_string_escaped())
            .ok_or(INVALID_JS_TYPE.into())
    }
}

impl IntoJs for Scalar {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        match self {
            Scalar::Null => Ok(JsValue::Null),
            Scalar::Boolean(v) => Ok(JsValue::Boolean(v)),
            Scalar::String(v) => Ok(JsValue::String(v.into())),
            Scalar::Float(v) => Ok(JsValue::Rational(v)),
            Scalar::Integer(v) => Ok(JsValue::Integer(v)),
            Scalar::Buffer(v) => bytes_to_jsuint8array(v, context),
            Scalar::Entity(v) => v.into_js(context),
        }
    }
}

impl FromJs for Scalar {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        if value.is_null() {
            Ok(Scalar::Null)
        } else if value.is_boolean() {
            Ok(Scalar::Boolean(value.as_boolean().ok_or(INVALID_JS_TYPE)?))
        } else if value.is_string() {
            Ok(Scalar::String(
                value
                    .as_string()
                    .ok_or(INVALID_JS_TYPE)?
                    .to_std_string_escaped(),
            ))
        } else if value.is_integer() {
            Ok(Scalar::Integer(
                value.to_i32(context).map_err(format_error)?,
            ))
        } else if value.is_double() {
            Ok(Scalar::Float(value.as_number().ok_or(INVALID_JS_TYPE)?))
        } else if value.is_object() {
            let object = value.as_object().ok_or(INVALID_JS_TYPE)?;
            if object.is_ordinary() {
                Ok(Scalar::Entity(Entity::from_js(value, context)?))
            } else if let Some(mut buffer) = object.downcast_mut::<ArrayBuffer>() {
                if let Some(inner) = buffer.detach(&JsValue::undefined()).map_err(format_error)? {
                    Ok(Scalar::Buffer(inner))
                } else {
                    // TODO discover this scenario
                    Ok(Scalar::Buffer(vec![]))
                }
            } else {
                Err(INVALID_JS_TYPE.into())
            }
        } else {
            Err(INVALID_JS_TYPE.into())
        }
    }
}

impl IntoJs for Vec<(String, Scalar)> {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        let object = JsObject::default();
        for (key, value) in self {
            object.insert_property(
                js_string!(key),
                PropertyDescriptor::builder()
                    .configurable(false)
                    .enumerable(true)
                    .writable(false)
                    .value(value.into_js(context)?)
                    .build(),
            );
        }
        Ok(JsValue::Object(object))
    }
}

/// Several return tupes are represented as arrays in
/// JS, and tuples in Rust.
///
/// `init` -> (State, RangeQuery)
/// `step` -> (State, Vec<Instruction>)
impl<A, B> FromJs for (A, B)
where
    A: FromJs,
    B: FromJs,
{
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let array = js_value_to_js_array(value)?;
        Ok((
            A::from_js(array.at(0, context).map_err(format_error)?, context)?,
            B::from_js(array.at(1, context).map_err(format_error)?, context)?,
        ))
    }
}

impl IntoJs for Datom {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        let out = js_object!(context, {
            "entity" => self.entity,
            "attribute" => self.attribute,
            "value" => self.value,
            "cause" => self.cause
        })?;
        Ok(out)
    }
}

impl FromJs for Fact {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let obj = value.as_object().ok_or(INVALID_JS_TYPE)?;
        let entity = Entity::from_js(
            obj.get(js_string!("entity"), context)
                .map_err(format_error)?,
            context,
        )?;
        let attribute = String::from_js(
            obj.get(js_string!("attribute"), context)
                .map_err(format_error)?,
            context,
        )?;
        let value = Scalar::from_js(
            obj.get(js_string!("value"), context)
                .map_err(format_error)?,
            context,
        )?;
        Ok(Fact {
            entity,
            attribute,
            value,
        })
    }
}

impl IntoJs for State {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        let string_state = String::from_utf8(self).map_err(format_error)?;
        let js_string = JsValue::String(js_string!(string_state));
        json_parse(js_string, context)
    }
}

impl FromJs for State {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let parsed = json_stringify(value, context)?
            .as_string()
            .ok_or(INVALID_JS_TYPE)?
            .to_std_string_escaped();
        Ok(parsed.into())
    }
}

impl IntoJs for Entity {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        js_object!(context, {
            "id" => self.id
        })
    }
}

impl FromJs for Entity {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let obj = value.as_object().ok_or(INVALID_JS_TYPE)?;
        let id = obj
            .get(js_string!("id"), context)
            .map_err(format_error)?
            .as_string()
            .map(|s| s.to_std_string_escaped())
            .ok_or(INVALID_JS_TYPE)?;
        Ok(Entity { id })
    }
}

impl IntoJs for String {
    fn into_js(self, _: &mut Context) -> Result<JsValue, String> {
        Ok(JsValue::String(js_string!(self)))
    }
}

/*
impl TryFrom<(&str, Scalar)> for Instruction {
    type Error = String;
    fn try_from(value: (&str, Scalar)) -> Result<Self, Self::Error> {
        match value.0 {
            OP_ASSERT => Ok(Instruction::Assert(value.1)),
            OP_RETRACT => Ok(Instruction::Retract(value.1)),
            OP_IMPORT => Ok(Instruction::Import(value.1)),
            e => Err(format!("Invalid instruction '{}'.", e)),
        }
    }
}

impl From<Instruction> for (&str, Scalar) {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::Assert(v) => (OP_ASSERT, v),
            Instruction::Retract(v) => (OP_RETRACT, v),
            Instruction::Import(v) => (OP_IMPORT, v),
        }
    }
}

// not needed?
impl IntoJs for Instruction {
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        let (op, value) = self.into();
        js_object!(context, {
            op => value
        })
    }
}
*/

impl<T> IntoJs for Vec<T>
where
    T: IntoJs,
{
    fn into_js(self, context: &mut Context) -> Result<JsValue, String> {
        let out = JsArray::new(context);
        for item in self {
            out.push(item.into_js(context)?, context)
                .map_err(format_error)?;
        }
        Ok(out.into())
    }
}

impl<T> FromJs for Vec<T>
where
    T: FromJs,
{
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let array = js_value_to_js_array(value)?;
        let length = array.length(context).map_err(format_error)?;
        let mut out = vec![];
        for i in 0..length {
            let value = T::from_js(array.at(i as i64, context).map_err(format_error)?, context)?;
            out.push(value);
        }
        Ok(out)
    }
}

impl FromJs for Instruction {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let obj = value.as_object().ok_or(INVALID_JS_TYPE)?;

        match (
            obj.get(js_string!(OP_ASSERT), context)
                .map_err(format_error)?,
            obj.get(js_string!(OP_RETRACT), context)
                .map_err(format_error)?,
            obj.get(js_string!(OP_IMPORT), context)
                .map_err(format_error)?,
        ) {
            (JsValue::Object(obj), JsValue::Undefined, JsValue::Undefined) => Ok(
                Instruction::Assert(Fact::from_js(JsValue::from(obj), context)?),
            ),
            (JsValue::Undefined, JsValue::Object(obj), JsValue::Undefined) => Ok(
                Instruction::Retract(Fact::from_js(JsValue::from(obj), context)?),
            ),
            (JsValue::Undefined, JsValue::Undefined, JsValue::Object(_)) => Ok(Instruction::Import),
            _ => Err(INVALID_JS_TYPE.into()),
        }
    }
}

impl FromJs for RangeQuery {
    fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
        let obj = value.as_object().ok_or(INVALID_JS_TYPE)?;
        match (
            obj.get(js_string!(RANGE_QUERY_BY_ENTITY), context)
                .map_err(format_error)?,
            obj.get(js_string!(RANGE_QUERY_BY_ATTRIBUTE), context)
                .map_err(format_error)?,
            obj.get(js_string!(RANGE_QUERY_BY_VALUE), context)
                .map_err(format_error)?,
        ) {
            (JsValue::Object(obj), JsValue::Undefined, JsValue::Undefined) => Ok(
                RangeQuery::Entity(EntityRangeQuery::from_js(JsValue::Object(obj), context)?),
            ),
            (JsValue::Undefined, JsValue::Object(obj), JsValue::Undefined) => Ok(
                RangeQuery::Attribute(AttributeRangeQuery::from_js(JsValue::Object(obj), context)?),
            ),
            (JsValue::Undefined, JsValue::Undefined, JsValue::Object(obj)) => Ok(
                RangeQuery::Value(ValueRangeQuery::from_js(JsValue::Object(obj), context)?),
            ),
            _ => Err(INVALID_JS_TYPE.into()),
        }
    }
}

macro_rules! range_query_type_from_js {
    ($query_type:ty, $entity_type:ty, $attr_type:ty, $value_type:ty) => {
        impl FromJs for $query_type {
            fn from_js(value: JsValue, context: &mut Context) -> Result<Self, String> {
                let obj = value.as_object().ok_or(INVALID_JS_TYPE)?;
                Ok(Self {
                    entity: <$entity_type>::from_js(
                        obj.get(js_string!("entity"), context)
                            .map_err(format_error)?,
                        context,
                    )?,
                    attribute: <$attr_type>::from_js(
                        obj.get(js_string!("attribute"), context)
                            .map_err(format_error)?,
                        context,
                    )?,
                    value: <$value_type>::from_js(
                        obj.get(js_string!("value"), context)
                            .map_err(format_error)?,
                        context,
                    )?,
                })
            }
        }
    };
}

range_query_type_from_js!(EntityRangeQuery, Entity, Option<String>, Option<Scalar>);
range_query_type_from_js!(AttributeRangeQuery, Option<Entity>, String, Option<Scalar>);
range_query_type_from_js!(ValueRangeQuery, Option<Entity>, Option<String>, Scalar);

macro_rules! into_scalar {
    ($rust_type:ty, $scalar_type: expr) => {
        impl From<$rust_type> for Scalar {
            fn from(value: $rust_type) -> Self {
                $scalar_type(value)
            }
        }
    };
}

impl From<()> for Scalar {
    fn from(_: ()) -> Self {
        Scalar::Null
    }
}

into_scalar!(bool, Scalar::Boolean);
into_scalar!(String, Scalar::String);
into_scalar!(f64, Scalar::Float);
into_scalar!(i32, Scalar::Integer);
into_scalar!(Vec<u8>, Scalar::Buffer);
into_scalar!(Entity, Scalar::Entity);

fn js_value_to_js_array(value: JsValue) -> Result<JsArray, String> {
    let obj = value.as_object().ok_or(INVALID_JS_TYPE)?.to_owned();
    JsArray::from_object(obj).map_err(format_error)
}

fn bytes_to_jsuint8array(bytes: Vec<u8>, context: &mut Context) -> Result<JsValue, String> {
    let buffer = JsArrayBuffer::from_byte_block(bytes, context).map_err(format_error)?;
    let array = JsUint8Array::from_array_buffer(buffer, context).map_err(format_error)?;
    Ok(array.into())
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
