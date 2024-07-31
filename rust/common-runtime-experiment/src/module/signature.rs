use super::ModuleId;
use common_wit::Target;

pub struct ModuleSignature {
    pub target: Target,
    pub id: ModuleId,
}
