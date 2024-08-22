mod affinity;
pub use affinity::*;

mod body;
pub use body::*;

mod definition;
pub use definition::*;

mod interface;
pub use interface::*;

mod source_code;
pub use source_code::*;

mod context;
pub use context::*;

mod manager;
pub use manager::*;

mod factory;
pub use factory::*;

mod driver;
pub use driver::*;

mod id;
pub use id::*;

/// [Module] is implemented for types that represent a live, instantiated Module
pub trait Module: HasModuleContextMut {
    /// A [ModuleId] that may be used to look up the Wasm artifact that
    /// constitutes the [Module]'s substantive implementation
    fn id(&self) -> &ModuleId;
    /// A [ModuleInstanceId] uniquely identifies the instance of the [Module]
    /// among all other instances of it and other [Module]s.
    fn instance_id(&self) -> &ModuleInstanceId;
}
