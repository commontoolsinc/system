use crate::{
    ifc::IfcPolicy, CommonIfcError, Confidentiality, Context, HasLabelType, IfcLabel, Integrity,
    Label, Lattice, ModuleEnvironment, Result,
};
use std::collections::BTreeMap;

type PolicyMapInner<T> = BTreeMap<T, Context>;

/// Map of [Confidentiality] or [Integrity]
/// labels to the minimum allowed [Context].
///
/// Map is validated upon construction.
#[derive(Clone)]
struct PolicyMap<T: Lattice + HasLabelType + 'static>(PolicyMapInner<T>);

impl<T> std::ops::Deref for PolicyMap<T>
where
    T: Lattice + HasLabelType + 'static,
{
    type Target = PolicyMapInner<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> TryFrom<PolicyMapInner<T>> for PolicyMap<T>
where
    T: Lattice + HasLabelType + 'static,
{
    type Error = CommonIfcError;

    /// Constructs a new policy map, validating that
    /// all labels have defined requirements.
    fn try_from(map: PolicyMapInner<T>) -> Result<Self> {
        for label in T::iter() {
            if !map.contains_key(label) {
                return Err(CommonIfcError::PolicyMissingDefinition {
                    label_type: T::label_type(),
                    level: label.to_string(),
                });
            }
        }
        Ok(Self(map))
    }
}

/// Represents an invoker's data flow requirements
/// to verify an execution graph with regard to the provided
/// data input.
///
/// Internally, contains maps of [Confidentiality]
/// and [Integrity] levels to [Context] requirements.
#[derive(Clone)]
pub struct Policy {
    /// Map of confidentiality principals to the minimum
    /// required [Context] components.
    confidentiality_map: PolicyMap<Confidentiality>,
    /// Map of integrity principals to the minimum
    /// required [Context] components.
    integrity_map: PolicyMap<Integrity>,
}

impl Policy {
    /// Create a new [Policy], given a map of [Confidentiality]
    /// and [Integrity] labels to a minimum required [Context].
    pub fn new<C, I>(confidentiality_map: C, integrity_map: I) -> Result<Self>
    where
        C: Into<PolicyMapInner<Confidentiality>>,
        I: Into<PolicyMapInner<Integrity>>,
    {
        let confidentiality_map = confidentiality_map.into().try_into()?;
        let integrity_map = integrity_map.into().try_into()?;
        Ok(Self {
            confidentiality_map,
            integrity_map,
        })
    }

    /// Validate input against this policy, given a [Context].
    /// Placeholder until we have full graph validations integrated
    /// with runtime.
    /// TBD if this will be run at runtime for every data hop in
    /// an execution graph.
    pub fn validate_single<'a, I>(&'a self, input: I, ctx: &Context) -> Result<()>
    where
        I: IntoIterator<Item = (&'a String, &'a Label)>,
    {
        for (_, label) in input {
            self.check_context(label, ctx)?;
        }
        Ok(())
    }

    /// Create a [Policy] from defaults.
    ///
    /// Explicitly not using [Default] trait so that
    /// defaults go through the same validation.
    pub fn with_defaults() -> Result<Self> {
        let confidentiality_map = [
            (Confidentiality::Low, (ModuleEnvironment::Server,).into()),
            (Confidentiality::High, (ModuleEnvironment::Server,).into()),
        ];
        let integrity_map = [
            (Integrity::Low, (ModuleEnvironment::Server,).into()),
            (Integrity::High, (ModuleEnvironment::Server,).into()),
        ];

        Self::new(confidentiality_map, integrity_map)
    }
}

impl IfcPolicy for Policy {
    type Context = Context;
    type Label = Label;
    fn get_requirements(&self, label: &Self::Label) -> Result<(&Self::Context, &Self::Context)> {
        match (
            self.confidentiality_map.get(&label.confidentiality),
            self.integrity_map.get(&label.integrity),
        ) {
            (Some(conf), Some(int)) => Ok((conf, int)),
            (None, _) => Err(CommonIfcError::PolicyMissingDefinition {
                label_type: <Self::Label as IfcLabel>::Confidentiality::label_type(),
                level: label.confidentiality().to_string(),
            }),
            (_, None) => Err(CommonIfcError::PolicyMissingDefinition {
                label_type: <Self::Label as IfcLabel>::Integrity::label_type(),
                level: label.integrity().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModuleEnvironment::*;
    use common_tracing::common_tracing;

    #[test]
    #[common_tracing]
    fn it_validates_module_env() -> Result<()> {
        let input =
            BTreeMap::from([("in".into(), (Confidentiality::High, Integrity::High).into())]);

        let policy = Policy::with_defaults()?;
        assert!(policy.validate_single(&input, &(Server,).into()).is_ok());
        assert!(policy
            .validate_single(&input, &(WebBrowser,).into())
            .is_ok());

        // Private data only on BrowserClient
        let policy = Policy::new(
            BTreeMap::from([
                (Confidentiality::Low, (Server,).into()),
                (Confidentiality::High, (WebBrowser,).into()),
            ]),
            BTreeMap::from([
                (Integrity::Low, (Server,).into()),
                (Integrity::High, (Server,).into()),
            ]),
        )?;
        assert!(policy.validate_single(&input, &(Server,).into()).is_err());
        assert!(policy
            .validate_single(&input, &(WebBrowser,).into())
            .is_ok());

        Ok(())
    }
}
