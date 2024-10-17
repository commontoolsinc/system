use std::collections::HashMap;

use common_macros::NewType;
use common_protos::formula as proto;

#[allow(missing_docs)]
pub mod virtual_module {
    wasmtime::component::bindgen!({
        world: "virtual-module",
        path: "../../wit/common/formula/wit",
        async: true
    });
}

pub use virtual_module::{
    exports::common::formula::module::{
        AttributeRangeQuery, Datom, Entity, EntityRangeQuery, Fact, Guest, Instruction, RangeQuery,
        Scalar, State, ValueRangeQuery,
    },
    VirtualModule,
};

use crate::CommonRuntimeError;

/// Map of [String] to [Scalar].
#[derive(NewType, Default, Clone, Debug)]
pub struct ScalarMap(Vec<(String, Scalar)>);

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<Entity> for proto::Entity {
    fn from(value: Entity) -> Self {
        proto::Entity { id: value.id }
    }
}

impl From<proto::Entity> for Entity {
    fn from(value: proto::Entity) -> Self {
        Entity { id: value.id }
    }
}

impl From<Fact> for proto::Fact {
    fn from(value: Fact) -> Self {
        proto::Fact {
            entity: Some(value.entity.into()),
            attribute: value.attribute,
            value: Some(value.value.into()),
        }
    }
}

impl TryFrom<proto::Fact> for Fact {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::Fact) -> Result<Self, Self::Error> {
        Ok(Fact {
            entity: value.entity.ok_or(CommonRuntimeError::InvalidValue)?.into(),
            attribute: value.attribute,
            value: value
                .value
                .ok_or(CommonRuntimeError::InvalidValue)?
                .try_into()?,
        })
    }
}

impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
            && self.attribute == other.attribute
            && self.value == other.value
    }
}

impl From<Datom> for proto::Datom {
    fn from(value: Datom) -> Self {
        proto::Datom {
            entity: Some(value.entity.into()),
            attribute: value.attribute,
            value: Some(value.value.into()),
            cause: Some(value.cause.into()),
        }
    }
}

impl TryFrom<proto::Datom> for Datom {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::Datom) -> Result<Self, Self::Error> {
        Ok(Datom {
            entity: value.entity.ok_or(CommonRuntimeError::InvalidValue)?.into(),
            attribute: value.attribute,
            value: value
                .value
                .ok_or(CommonRuntimeError::InvalidValue)?
                .try_into()?,
            cause: value.cause.ok_or(CommonRuntimeError::InvalidValue)?.into(),
        })
    }
}

impl From<Instruction> for proto::Instruction {
    fn from(value: Instruction) -> Self {
        match value {
            Instruction::Assert(v) => proto::Instruction {
                kind: proto::InstructionKind::Assert.into(),
                value: Some(v.into()),
            },
            Instruction::Retract(v) => proto::Instruction {
                kind: proto::InstructionKind::Retract.into(),
                value: Some(v.into()),
            },
            Instruction::Import => proto::Instruction {
                kind: proto::InstructionKind::Import.into(),
                value: None,
            },
        }
    }
}

impl TryFrom<proto::Instruction> for Instruction {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::Instruction) -> Result<Self, Self::Error> {
        let fact = Fact::try_from(value.value.ok_or(CommonRuntimeError::InvalidValue)?)?;
        let kind = proto::InstructionKind::try_from(value.kind)
            .map_err(|_| CommonRuntimeError::InvalidValue)?;
        Ok(match kind {
            proto::InstructionKind::Assert => Instruction::Assert(fact),
            proto::InstructionKind::Retract => Instruction::Retract(fact),
            proto::InstructionKind::Import => Instruction::Import,
        })
    }
}

impl TryFrom<HashMap<String, proto::Scalar>> for ScalarMap {
    type Error = CommonRuntimeError;
    fn try_from(proto: HashMap<String, proto::Scalar>) -> Result<Self, Self::Error> {
        let mut map = Vec::new();
        for (key, value) in proto.into_iter() {
            map.push((key, Scalar::try_from(value)?));
        }
        Ok(Self(map))
    }
}

impl From<ScalarMap> for HashMap<String, proto::Scalar> {
    fn from(value: ScalarMap) -> Self {
        let mut map = HashMap::new();
        for (key, value) in value.into_inner() {
            map.insert(key, proto::Scalar::from(value));
        }
        map
    }
}

impl From<Scalar> for proto::Scalar {
    fn from(value: Scalar) -> Self {
        let variant = match value {
            Scalar::Null => proto::scalar::Variant::Null(false),
            Scalar::Boolean(i) => proto::scalar::Variant::Boolean(i),
            Scalar::String(i) => proto::scalar::Variant::String(i),
            Scalar::Integer(i) => proto::scalar::Variant::Integer(i),
            Scalar::Float(i) => proto::scalar::Variant::Float(i),
            Scalar::Buffer(i) => proto::scalar::Variant::Buffer(i),
            Scalar::Entity(i) => proto::scalar::Variant::Entity(i.into()),
        };
        proto::Scalar {
            variant: Some(variant),
        }
    }
}

impl TryFrom<proto::Scalar> for Scalar {
    type Error = CommonRuntimeError;

    fn try_from(value: proto::Scalar) -> Result<Self, Self::Error> {
        let value = value.variant.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match value {
            proto::scalar::Variant::Null(_) => Scalar::Null,
            proto::scalar::Variant::String(string) => Scalar::String(string),
            proto::scalar::Variant::Integer(number) => Scalar::Integer(number),
            proto::scalar::Variant::Float(number) => Scalar::Float(number),
            proto::scalar::Variant::Boolean(boolean) => Scalar::Boolean(boolean),
            proto::scalar::Variant::Buffer(buffer) => Scalar::Buffer(buffer),
            proto::scalar::Variant::Entity(e) => Scalar::Entity(e.into()),
        })
    }
}

impl From<EntityRangeQuery> for proto::EntityRangeQuery {
    fn from(value: EntityRangeQuery) -> Self {
        proto::EntityRangeQuery {
            entity: Some(value.entity.into()),
            attribute: value.attribute,
            value: value.value.map(Into::into),
        }
    }
}

impl TryFrom<proto::EntityRangeQuery> for EntityRangeQuery {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::EntityRangeQuery) -> Result<Self, Self::Error> {
        Ok(EntityRangeQuery {
            entity: value.entity.ok_or(CommonRuntimeError::InvalidValue)?.into(),
            attribute: value.attribute,
            value: value
                .value
                .map_or_else(|| Ok(None), |inner| inner.try_into().map(Some))?,
        })
    }
}

impl From<AttributeRangeQuery> for proto::AttributeRangeQuery {
    fn from(value: AttributeRangeQuery) -> Self {
        proto::AttributeRangeQuery {
            entity: value.entity.map(Into::into),
            attribute: value.attribute,
            value: value.value.map(Into::into),
        }
    }
}

impl TryFrom<proto::AttributeRangeQuery> for AttributeRangeQuery {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::AttributeRangeQuery) -> Result<Self, Self::Error> {
        Ok(AttributeRangeQuery {
            entity: value.entity.map(Into::into),
            attribute: value.attribute,
            value: value
                .value
                .map_or_else(|| Ok(None), |inner| inner.try_into().map(Some))?,
        })
    }
}

impl From<ValueRangeQuery> for proto::ValueRangeQuery {
    fn from(value: ValueRangeQuery) -> Self {
        proto::ValueRangeQuery {
            entity: value.entity.map(Into::into),
            attribute: value.attribute,
            value: Some(value.value.into()),
        }
    }
}

impl TryFrom<proto::ValueRangeQuery> for ValueRangeQuery {
    type Error = CommonRuntimeError;
    fn try_from(value: proto::ValueRangeQuery) -> Result<Self, Self::Error> {
        Ok(ValueRangeQuery {
            entity: value.entity.map(Into::into),
            attribute: value.attribute,
            value: value
                .value
                .ok_or(CommonRuntimeError::InvalidValue)?
                .try_into()?,
        })
    }
}

impl TryFrom<proto::RangeQuery> for RangeQuery {
    type Error = CommonRuntimeError;

    fn try_from(value: proto::RangeQuery) -> Result<Self, Self::Error> {
        let value = value.variant.ok_or(CommonRuntimeError::InvalidValue)?;
        Ok(match value {
            proto::range_query::Variant::Entity(inner) => RangeQuery::Entity(inner.try_into()?),
            proto::range_query::Variant::Attribute(inner) => {
                RangeQuery::Attribute(inner.try_into()?)
            }
            proto::range_query::Variant::Value(inner) => RangeQuery::Value(inner.try_into()?),
        })
    }
}

impl From<RangeQuery> for proto::RangeQuery {
    fn from(value: RangeQuery) -> Self {
        let variant = match value {
            RangeQuery::Entity(inner) => proto::range_query::Variant::Entity(inner.into()),
            RangeQuery::Attribute(inner) => proto::range_query::Variant::Attribute(inner.into()),
            RangeQuery::Value(inner) => proto::range_query::Variant::Value(inner.into()),
        };
        proto::RangeQuery {
            variant: Some(variant),
        }
    }
}

macro_rules! range_query_partial_eq {
    ($query_type:ty) => {
        impl PartialEq for $query_type {
            fn eq(&self, other: &Self) -> bool {
                self.entity == other.entity
                    && self.attribute == other.attribute
                    && self.value == other.value
            }
        }
    };
}

range_query_partial_eq!(EntityRangeQuery);
range_query_partial_eq!(AttributeRangeQuery);
range_query_partial_eq!(ValueRangeQuery);

impl PartialEq for RangeQuery {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RangeQuery::Entity(a), RangeQuery::Entity(b)) => a == b,
            (RangeQuery::Attribute(a), RangeQuery::Attribute(b)) => a == b,
            (RangeQuery::Value(a), RangeQuery::Value(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Instruction::Assert(a), Instruction::Assert(b)) => a == b,
            (Instruction::Retract(a), Instruction::Retract(b)) => a == b,
            (Instruction::Import, Instruction::Import) => true,
            _ => false,
        }
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Scalar::Null, Scalar::Null) => true,
            (Scalar::Boolean(a), Scalar::Boolean(b)) => a == b,
            (Scalar::Buffer(a), Scalar::Buffer(b)) => a == b,
            (Scalar::Integer(a), Scalar::Integer(b)) => a == b,
            (Scalar::String(a), Scalar::String(b)) => a == b,
            (Scalar::Float(a), Scalar::Float(b)) => a == b,
            (Scalar::Entity(a), Scalar::Entity(b)) => a == b,
            _ => false,
        }
    }
}

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
