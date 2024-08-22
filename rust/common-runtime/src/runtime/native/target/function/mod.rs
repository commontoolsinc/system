//! Substantive implementation of a local `common:function/virtual-module` for the
//! [crate::NativeRuntime]

mod factory;
pub use factory::*;

mod module;
pub use module::*;

mod context;
pub use context::*;
