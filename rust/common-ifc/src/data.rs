use crate::{Confidentiality, IfcError, Integrity, Label};
use common_protos::common as proto;
use std::str::FromStr;

/// The data that gets passed between runtime modules,
/// containing the underlying `T` and its confidentiality
/// and integrity [Label].
#[derive(PartialEq, Clone, Debug)]
pub struct Data<T> {
    /// The inner value.
    pub value: T,
    /// [Label] representing confidentiality and integrity
    /// of `value`.
    pub label: Label,
}

impl<T> Data<T> {
    /// Creates a [Data] from a value `T` using the
    /// strictest labels: the most confidential, and the
    /// least integrity.
    pub fn with_strict_labels(value: T) -> Self {
        Data {
            value,
            label: Label::default(),
        }
    }
}

impl<T> TryFrom<proto::LabeledData> for Data<T>
where
    T: TryFrom<proto::Value>,
{
    type Error = IfcError;
    fn try_from(data: proto::LabeledData) -> Result<Self, Self::Error> {
        Ok(Data {
            value: data
                .value
                .ok_or(IfcError::Conversion)?
                .try_into()
                .map_err(|_| IfcError::Conversion)?,
            label: (
                Confidentiality::from_str(&data.confidentiality)?,
                Integrity::from_str(&data.integrity)?,
            )
                .into(),
        })
    }
}

impl<T> From<Data<T>> for proto::LabeledData
where
    T: Into<proto::Value>,
{
    fn from(data: Data<T>) -> Self {
        proto::LabeledData {
            value: Some(data.value.into()),
            confidentiality: data.label.confidentiality.to_string(),
            integrity: data.label.integrity.to_string(),
        }
    }
}

impl<T> From<(T, Confidentiality, Integrity)> for Data<T> {
    fn from(data: (T, Confidentiality, Integrity)) -> Self {
        Data {
            value: data.0,
            label: (data.1, data.2).into(),
        }
    }
}

impl<T> From<(T, Label)> for Data<T> {
    fn from(data: (T, Label)) -> Self {
        Data {
            value: data.0,
            label: data.1,
        }
    }
}
