use common_macros::NewType;
use common_wit::Target;

use crate::{module::affinity::Affinity, CommonRuntimeError};

use super::ModuleDefinition;

/// A newtype over a [ModuleDefinition]; it can only be constructed for
/// definitions whose target is [Target::CommonFunction].and whose affinity
/// allows for local instantiation.
#[derive(NewType)]
#[new_type(skip(From))]
pub struct FunctionDefinition(ModuleDefinition);

impl TryFrom<ModuleDefinition> for FunctionDefinition {
    type Error = CommonRuntimeError;

    fn try_from(value: ModuleDefinition) -> Result<Self, Self::Error> {
        if Target::CommonFunction != value.target {
            Err(CommonRuntimeError::PreparationFailed(format!(
                "Expected {}, got {}",
                Target::CommonFunction,
                value.target
            )))
        } else {
            match value.affinity {
                Affinity::LocalOnly | Affinity::PrefersLocal | Affinity::PrefersRemote => {
                    Ok(FunctionDefinition(value))
                }
                Affinity::RemoteOnly => Err(CommonRuntimeError::PreparationFailed(
                    "Cannot instantiate local module with remote-only affinity".to_string(),
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use anyhow::Result;
    use common_wit::Target;

    use crate::{Affinity, ModuleDefinition};

    use super::FunctionDefinition;

    #[test]
    fn it_ensures_local_affinity() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        FunctionDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::RemoteOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        assert!(FunctionDefinition::try_from(failing_module_definition).is_err());
    }

    #[test]
    fn it_ensures_common_function_target() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        FunctionDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        assert!(FunctionDefinition::try_from(failing_module_definition).is_err());
    }
}
