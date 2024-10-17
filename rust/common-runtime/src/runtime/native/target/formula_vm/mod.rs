//! Substantive implementation of a local `common:formula/virtual-module` for
//! the [crate::NativeRuntime]

mod bindings;
pub use bindings::*;

mod factory;
pub use factory::*;

mod module;
pub use module::*;

mod context;
pub use context::*;
