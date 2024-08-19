use common_macros::NewType;
use common_wit::Target;

use crate::{module::affinity::Affinity, CommonRuntimeError};

use super::ModuleDefinition;

/// Remote Function Definition
///
/// A newtype over a [ModuleDefinition]; it can only be constructed for
/// definitions whose target is [Target::CommonFunction].or [Target::CommonFunctionVm] and whose affinity
/// allows for remote instantiation.
#[derive(NewType)]
#[new_type(skip(From))]
pub struct RemoteFunctionDefinition(ModuleDefinition);

impl TryFrom<ModuleDefinition> for RemoteFunctionDefinition {
    type Error = CommonRuntimeError;

    fn try_from(value: ModuleDefinition) -> Result<Self, Self::Error> {
        match value.target {
            Target::CommonFunction | Target::CommonFunctionVm => match value.affinity {
                Affinity::RemoteOnly | Affinity::PrefersLocal | Affinity::PrefersRemote => {
                    Ok(RemoteFunctionDefinition(value))
                }
                Affinity::LocalOnly => Err(CommonRuntimeError::PreparationFailed(
                    "Cannot instantiate remote module with local-only affinity".to_string(),
                )),
            },
            // ALLOW: Future additions to the `Target` enum should fall through here by default
            #[allow(unreachable_patterns)]
            unexpected => Err(CommonRuntimeError::PreparationFailed(format!(
                "Expected {} or {}, got {}",
                Target::CommonFunction,
                Target::CommonFunctionVm,
                unexpected
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use common_wit::Target;

    use crate::{Affinity, ModuleDefinition};

    use super::RemoteFunctionDefinition;

    #[test]
    fn it_ensures_remote_affinity() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::RemoteOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        RemoteFunctionDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        assert!(RemoteFunctionDefinition::try_from(failing_module_definition).is_err());
    }

    #[test]
    fn it_allows_common_function_or_common_function_vm_target() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::RemoteOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        RemoteFunctionDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::RemoteOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        RemoteFunctionDefinition::try_from(failing_module_definition).unwrap();
    }
}
