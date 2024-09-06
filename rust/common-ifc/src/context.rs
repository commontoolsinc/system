use crate::{CommonIfcError, Result};

#[cfg(doc)]
use crate::Policy;

/// Environment a module is being evaluated in,
/// ordered from least to most "private".
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum ModuleEnvironment {
    /// Confidential compute environment.
    Server,
    /// On a web browser client.
    WebBrowser,
}

/// Represents an execution environment of a module.
///
/// Used in [`Policy`] as requirements for
/// the minimum level needed to execute a module,
/// validating against the actual [`Context`] during
/// execution.
#[derive(Debug, Clone)]
pub struct Context {
    /// Minimum allowed module environment.
    pub environment: ModuleEnvironment,
}

impl Context {
    /// Ensures the provided [`Context`] surpasses
    /// the threshold for all of this context's requirements.
    pub fn validate(&self, ctx: &Context) -> Result<()> {
        if self.environment > ctx.environment {
            return Err(CommonIfcError::InvalidEnvironment);
        }
        Ok(())
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
    fn it_validates_context() -> Result<()> {
        let server = Context::from((Server,));
        let browser = Context::from((WebBrowser,));

        server.validate(&server)?;
        server.validate(&browser)?;
        browser.validate(&browser)?;
        assert!(browser.validate(&server).is_err());
        Ok(())
    }
}
