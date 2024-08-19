use super::ModuleInstanceId;
use async_trait::async_trait;

/// [ModuleManager] is implemented for each distinctive [crate::Module] type
/// that is tracked by the implementor. The implementor may then be used to keep
/// instances of [crate::Module]s alive.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ModuleManager<T> {
    /// Track a [crate::Module] by retaining a reference to it
    async fn add(&self, module_instance: T) -> ModuleInstanceId;
    /// Look up a tracked [crate::Module] by [ModuleInstanceId]
    async fn get(&self, id: &ModuleInstanceId) -> Option<T>;
    /// Take a tracked [crate::Module]; the [ModuleManager] will no longer
    /// retain a reference to it
    async fn take(&self, id: ModuleInstanceId) -> Option<T>;
}
