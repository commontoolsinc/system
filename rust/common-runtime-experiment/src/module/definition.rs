use super::{ModuleBody, ModuleIoShape};

pub struct Module<Target, Affinity>
where
    Target: Into<common_wit::Target>,
{
    pub target: Target,
    pub affinity: Affinity,

    pub body: ModuleBody,
    pub io_shape: ModuleIoShape,
}
