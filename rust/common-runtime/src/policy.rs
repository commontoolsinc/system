use crate::{CommonRuntimeError, InputOutput};
use common_ifc::{Context as IfcContext, Policy};

/// A type that wraps an inner value that has been validated against some [Policy].
/// A [Validated] can only be created through a fallible step in which the wrapped
/// value is validated against the [Policy].
pub struct Validated<T> {
    policy: Policy,
    inner: T,
}

impl<T> Validated<T> {
    /// Unwrap the inner value and return it
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// The [Policy] that was used to validate the wrapped value
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

impl<Io> TryFrom<(Policy, &IfcContext, Io)> for Validated<Io>
where
    Io: InputOutput,
{
    type Error = CommonRuntimeError;

    fn try_from((policy, context, io): (Policy, &IfcContext, Io)) -> Result<Self, Self::Error> {
        policy.validate(io.input().iter(), context)?;
        Ok(Self { policy, inner: io })
    }
}

impl<Io> TryFrom<(&Policy, &IfcContext, Io)> for Validated<Io>
where
    Io: InputOutput,
{
    type Error = CommonRuntimeError;

    fn try_from((policy, context, io): (&Policy, &IfcContext, Io)) -> Result<Self, Self::Error> {
        Self::try_from((policy.clone(), context, io))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Result;
    use common_ifc::{
        Confidentiality, Context as IfcContext, Data, Integrity, ModuleEnvironment, Policy,
    };

    use crate::{BasicIo, IoData, IoValues, Value};

    use super::Validated;

    #[test]
    fn it_accepts_io_that_aligns_with_the_policy() -> Result<()> {
        let policy = Policy::with_defaults()?;
        let input_values = IoValues::from(BTreeMap::from([(
            "foo".to_owned(),
            Value::String("bar".into()),
        )]));
        let io = BasicIo::from_initial_state(input_values, Default::default());
        let context = IfcContext {
            environment: ModuleEnvironment::Server,
        };

        let _ = Validated::try_from((policy, &context, io))?;

        Ok(())
    }

    #[test]
    fn it_rejects_io_that_does_not_align_with_the_policy() -> Result<()> {
        let policy = Policy::new(
            BTreeMap::from([
                (Confidentiality::Public, (ModuleEnvironment::Server,).into()),
                (
                    Confidentiality::Private,
                    (ModuleEnvironment::WebBrowser,).into(),
                ),
            ]),
            BTreeMap::from([
                (Integrity::Low, (ModuleEnvironment::Server,).into()),
                (Integrity::High, (ModuleEnvironment::Server,).into()),
            ]),
        )?;
        let input_data = IoData::new(BTreeMap::from([(
            "foo".to_owned(),
            Data::from((
                Value::String("bar".to_owned()),
                Confidentiality::Private,
                Integrity::High,
            )),
        )]));
        let io = BasicIo::new(input_data, Default::default());
        let context = IfcContext {
            environment: ModuleEnvironment::Server,
        };

        assert!(Validated::try_from((policy, &context, io)).is_err());

        Ok(())
    }
}
