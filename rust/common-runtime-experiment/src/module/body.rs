use super::{ModuleId, SourceCodeCollection};

pub enum ModuleBody {
    Id(ModuleId),
    SourceCode(SourceCodeCollection),
}
