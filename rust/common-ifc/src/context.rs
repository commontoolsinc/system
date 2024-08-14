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
/// Used in [Policy] as requirements for
/// the minimum level needed to execute a module,
/// validating against the actual [Context] during
/// execution.
pub struct Context {
    /// Minimum allowed module environment.
    pub env: ModuleEnvironment,
}

impl Context {
    /// Ensures the provided [Context] surpasses
    /// the threshold for all of this context's requirements.
    pub fn validate(&self, ctx: &Context, input_name: &str) -> Result<()> {
        if self.env > ctx.env {
            return Err(CommonIfcError::InvalidEnvironment(input_name.into()));
        }
        Ok(())
    }
}

impl From<(ModuleEnvironment,)> for Context {
    fn from(value: (ModuleEnvironment,)) -> Self {
        Context { env: value.0 }
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
        let name = "input";

        server.validate(&server, name)?;
        server.validate(&browser, name)?;
        browser.validate(&browser, name)?;
        assert!(browser.validate(&server, name).is_err());
        Ok(())
    }
}
