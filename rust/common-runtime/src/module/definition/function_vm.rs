use common_macros::NewType;
use common_wit::Target;

use crate::{Affinity, CommonRuntimeError, ContentType, ModuleBody};

use super::ModuleDefinition;

/// Function VM Definition
///
/// A newtype over a [`ModuleDefinition`]; it can only be constructed for
/// definitions whose target is [Target::CommonFunctionVm], whose affinity
/// allows for local instantiation and whose body is [ModuleBody::SourceCode].
#[derive(NewType)]
#[new_type(skip(From))]
pub struct FunctionVmDefinition(ModuleDefinition);

impl FunctionVmDefinition {
    /// Get the [`ContentType`] associated with the with the source code of this
    /// [`ModuleDefinition`]
    pub fn content_type(&self) -> Result<ContentType, CommonRuntimeError> {
        if let ModuleBody::SourceCode(source_code_collection) = &self.0.body {
            if let Some((_, source_code)) = source_code_collection.iter().next() {
                return Ok(source_code.content_type);
            };
        };
        Err(CommonRuntimeError::InvalidModuleSource(
            "Source code content-type was required but not specified".to_string(),
        ))
    }
}

impl TryFrom<ModuleDefinition> for FunctionVmDefinition {
    type Error = CommonRuntimeError;

    fn try_from(value: ModuleDefinition) -> Result<Self, Self::Error> {
        if Target::CommonFunctionVm != value.target {
            Err(CommonRuntimeError::PreparationFailed(format!(
                "Expected {}, got {}",
                Target::CommonFunctionVm,
                value.target
            )))
        } else {
            match value.affinity {
                Affinity::LocalOnly | Affinity::PrefersLocal | Affinity::PrefersRemote => {
                    match value.body {
                        crate::ModuleBody::Signature(_) => {
                            Err(CommonRuntimeError::PreparationFailed(
                                "Module signature body is not supported for virtual modules"
                                    .to_string(),
                            ))
                        }
                        crate::ModuleBody::SourceCode(_) => Ok(FunctionVmDefinition(value)),
                    }
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

    use crate::{Affinity, ModuleBody, ModuleDefinition, ModuleId};

    use super::FunctionVmDefinition;

    #[test]
    fn it_ensures_local_affinity() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        FunctionVmDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::RemoteOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        assert!(FunctionVmDefinition::try_from(failing_module_definition).is_err());
    }

    #[test]
    fn it_ensures_common_function_vm_target() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: crate::ModuleBody::SourceCode(Default::default()),
        };

        FunctionVmDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: ModuleBody::SourceCode(Default::default()),
        };

        assert!(FunctionVmDefinition::try_from(failing_module_definition).is_err());
    }

    #[test]
    fn it_ensures_source_code_body() {
        let passing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: ModuleBody::SourceCode(Default::default()),
        };

        FunctionVmDefinition::try_from(passing_module_definition).unwrap();

        let failing_module_definition = ModuleDefinition {
            target: Target::CommonFunctionVm,
            affinity: Affinity::LocalOnly,
            inputs: Default::default(),
            outputs: Default::default(),
            body: ModuleBody::Signature(ModuleId::Base64("FAKE".into())),
        };

        assert!(FunctionVmDefinition::try_from(failing_module_definition).is_err());
    }
}
