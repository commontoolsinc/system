#![warn(missing_docs)]

//! Information flow control for Common runtime.
//!
//! [Data] wraps a value, tagging it with [Confidentiality]
//! and [Integrity] labels. A [Policy] contains a map
//! of these labels and their [Context] requirements,
//! describing conditions that must be met in order
//! to permit data flow.
//!
//! <https://en.wikipedia.org/wiki/Information_flow_(information_theory)#Information_flow_control>

mod context;
mod data;
mod error;
mod labels;
mod policy;

pub use common_macros::Lattice;
pub use context::{Context, ModuleEnvironment};
pub use data::Data;
pub use error::{IfcError, Result};
pub use labels::{Confidentiality, Integrity, Label, Lattice};
pub use policy::Policy;
