use crate::{CommonRuntimeError, ValueKind};

use super::{Module, ModuleBody, ModuleId, ModuleIoShape, SourceCode, SourceCodeCollection};

#[derive(Default)]
pub struct ModuleBuilder<Target, Affinity>
where
    Target: Into<common_wit::Target>,
{
    target: Option<Target>,
    affinity: Option<Affinity>,

    module_id: Option<ModuleId>,
    module_sources: SourceCodeCollection,
    io_shape: ModuleIoShape,
}

impl<Target, Affinity> ModuleBuilder<Target, Affinity>
where
    Target: Into<common_wit::Target>,
{
    pub fn target(mut self, target: Target) -> Self {
        self.target = Some(target);
        self
    }

    pub fn affinity(mut self, affinity: Affinity) -> Self {
        self.affinity = Some(affinity);
        self
    }

    pub fn input(mut self, key: String, value: ValueKind) -> Self {
        self.io_shape.input.insert(key, value);
        self
    }

    pub fn output(mut self, key: String, value: ValueKind) -> Self {
        self.io_shape.output.insert(key, value);
        self
    }

    pub fn source_code(mut self, name: String, source_code: SourceCode) -> Self {
        self.module_sources.insert(name, source_code);
        self
    }

    pub fn build(self) -> Result<Module<Target, Affinity>, CommonRuntimeError> {
        let body = if let Some(id) = self.module_id {
            ModuleBody::Id(id.clone())
        } else if !self.module_sources.is_empty() {
            ModuleBody::SourceCode(self.module_sources)
        } else {
            return Err(CommonRuntimeError::InvalidModuleSource(
                "Either a Module ID or at least one input source file must be provided".into(),
            ));
        };

        let target = if let Some(target) = self.target {
            target
        } else {
            return Err(CommonRuntimeError::InvalidModuleTarget(
                "A Target was not configured for the Common Module".into(),
            ));
        };

        let affinity = if let Some(affinity) = self.affinity {
            affinity
        } else {
            return Err(CommonRuntimeError::InvalidModuleAffinity(
                "An Affinity was not configured for the Common Module".into(),
            ));
        };

        let io_shape = self.io_shape;

        Ok(Module {
            target,
            affinity,
            body,
            io_shape,
        })
    }
}
