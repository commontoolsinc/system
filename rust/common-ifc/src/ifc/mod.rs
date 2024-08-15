//! Core traits and types for validating policy
//! over an execution graph.

mod core;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
mod validation;

pub use core::*;
