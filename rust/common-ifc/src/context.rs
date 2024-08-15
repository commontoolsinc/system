use crate::IfcContext;
use anyhow::anyhow;

#[cfg(doc)]
use crate::Policy;

/// Environment a module is being evaluated in,
/// ordered from least to most "private".
#[derive(strum::Display, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum ModuleEnvironment {
    /// Confidential compute environment.
    Server,
    /// On a web browser client.
    WebBrowser,
}

/// Represents an execution environment of a module.
///
/// Used in [Policy] as requirements for
/// the minimum level needed to execute a module,
/// validating against the actual [Context] during
/// execution.
#[derive(Debug, Clone)]
pub struct Context {
    /// Minimum allowed module environment.
    pub environment: ModuleEnvironment,
}

impl IfcContext for Context {
    type Error = anyhow::Error;
    fn validate(&self, context: &Self) -> ::std::result::Result<(), Self::Error> {
        match self.environment <= context.environment {
            true => Ok(()),
            false => Err(anyhow!(
                "Policy for {} does not allow {}.",
                self.environment,
                context.environment
            )),
        }
    }
}

impl From<(ModuleEnvironment,)> for Context {
    fn from(value: (ModuleEnvironment,)) -> Self {
        Context {
            environment: value.0,
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
    fn it_validates_context() {
        let server = Context::from((Server,));
        let browser = Context::from((WebBrowser,));

        assert!(server.validate(&server).is_ok());
        assert!(server.validate(&browser).is_ok());
        assert!(browser.validate(&browser).is_ok());
        assert!(browser.validate(&server).is_err());
    }
}
