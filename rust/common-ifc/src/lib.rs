#![warn(missing_docs)]

//! Information flow control for Common runtime.
//!
//! Data in the system is wrapped by a [`Label`], representing
//! its [`Confidentiality`] and [`Integrity`] levels.
//! A [`Policy`] contains a map of these labels and
//! their [`Context`] requirements, describing conditions
//! that must be met in order to permit data flow.
//!
//! <https://en.wikipedia.org/wiki/Information_flow_(information_theory)#Information_flow_control>

mod context;
mod error;
mod graph;
mod labels;
mod policy;

pub use common_macros::Lattice;
pub use context::{Context, ModuleEnvironment};
pub use error::{CommonIfcError, PolicyViolationSource, Result};
pub use graph::validate_graph;
pub use labels::{Confidentiality, Integrity, Label, LabelType, Lattice};
pub use policy::Policy;
