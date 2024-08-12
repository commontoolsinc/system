use crate::{
    Confidentiality, Context, Data, IfcError, Integrity, Label, Lattice, ModuleEnvironment, Result,
};
use std::collections::BTreeMap;

type PolicyMapInner<T> = BTreeMap<T, Context>;

/// Map of [Confidentiality] or [Integrity]
/// labels to the minimum allowed [Context].
///
/// Map is validated upon construction.
struct PolicyMap<T: Lattice + 'static>(PolicyMapInner<T>);

impl<T> std::ops::Deref for PolicyMap<T>
where
    T: Lattice + 'static,
{
    type Target = PolicyMapInner<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> TryFrom<PolicyMapInner<T>> for PolicyMap<T>
where
    T: Lattice + 'static,
{
    type Error = IfcError;

    /// Constructs a new policy map, validating that
    /// all labels have defined requirements.
    fn try_from(map: PolicyMapInner<T>) -> Result<Self> {
        for label in T::iter() {
            if !map.contains_key(label) {
                return Err(IfcError::InvalidPolicy(format!(
                    "No requirements defined for {label}"
                )));
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
    pub fn validate<'a, T, I>(&'a self, input: I, ctx: &Context) -> Result<()>
    where
        T: 'static,
        I: IntoIterator<Item = (&'a String, &'a Data<T>)>,
    {
        for (name, data) in input {
            let (conf_reqs, int_reqs) = self.get_requirements(&data.label)?;
            conf_reqs.validate(ctx, name)?;
            int_reqs.validate(ctx, name)?;
        }
        Ok(())
    }

    /// Create a [Policy] from defaults.
    ///
    /// Explicitly not using [Default] trait so that
    /// defaults go through the same validation.
    pub fn with_defaults() -> Result<Self> {
        use Confidentiality::*;
        use Integrity::*;
        use ModuleEnvironment::*;

        let confidentiality_map = [(Public, (Server,).into()), (Private, (Server,).into())];
        let integrity_map = [
            (LowIntegrity, (Server,).into()),
            (HighIntegrity, (Server,).into()),
        ];

        Self::new(confidentiality_map, integrity_map)
    }

    /// Returns the [Context] requirements defined in this policy
    /// for the given [Label].
    fn get_requirements(&self, label: &Label) -> Result<(&Context, &Context)> {
        match (
            self.confidentiality_map.get(&label.confidentiality),
            self.integrity_map.get(&label.integrity),
        ) {
            (Some(conf_reqs), Some(int_reqs)) => Ok((conf_reqs, int_reqs)),
            (None, _) => Err(IfcError::InvalidPolicy(format!(
                "Policy missing confidentiality label '{}'",
                label.confidentiality
            ))),
            (_, None) => Err(IfcError::InvalidPolicy(format!(
                "Policy missing integrity label '{}'",
                label.integrity
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Confidentiality::*, Integrity::*, ModuleEnvironment::*};
    use common_tracing::common_tracing;

    #[test]
    #[common_tracing]
    fn it_validates_module_env() -> Result<()> {
        let input = BTreeMap::from([("in".into(), Data::from(("data", Private, HighIntegrity)))]);

        let policy = Policy::with_defaults()?;
        assert!(policy.validate(&input, &(Server,).into()).is_ok());
        assert!(policy.validate(&input, &(WebBrowser,).into()).is_ok());

        // Private data only on BrowserClient
        let policy = Policy::new(
            BTreeMap::from([(Public, (Server,).into()), (Private, (WebBrowser,).into())]),
            BTreeMap::from([
                (LowIntegrity, (Server,).into()),
                (HighIntegrity, (Server,).into()),
            ]),
        )?;
        assert_eq!(
            policy.validate(&input, &(Server,).into()),
            Err(IfcError::InvalidEnvironment("in".into()))
        );
        assert!(policy.validate(&input, &(WebBrowser,).into()).is_ok());

        Ok(())
    }
}