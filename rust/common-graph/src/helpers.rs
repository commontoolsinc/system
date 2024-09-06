//! Helpers used in tests.

use crate::{CommonGraphError, GraphProcessorItem};
use std::fmt::Debug;
use thiserror::Error;

#[cfg(doc)]
use crate::GraphProcessor;

/// Number type used by [`Op`].
pub type OpNum = f32;

/// Errors for [`Op`] commands.
#[derive(Error, Debug)]
pub enum OpError {
    /// Invalid argument.
    #[error("Invalid argument.")]
    InvalidArgument,
    /// Node inner not available.
    #[error("Node inner not found.")]
    MissingInner,
    /// No "out" output port found.
    #[error("Node does not have 'out' output.")]
    MissingOutput,
    /// Non-descript error.
    #[error("{0}")]
    Other(String),
}

/// Math and logical operators used to create
/// computations in tests.
#[derive(Debug)]
pub enum Op {
    /// Adds `x` and `y`.
    Add,
    /// Subtracts `y` from `x`.
    Subtract,
    /// Identity op that always returns its `.0`.
    Identity(OpNum),
    /// Multiplies `x` by `y`.
    Multiply,
    /// Divides `x` by `y`.
    Divide,
    /// Finds the maximum value from all inputs
    Max,
    /// Finds the minimum value from all inputs
    Min,
    /// Returns `1` if `x` and `y` are equal, `0` otherwise.
    Eq,
    /// Returns `1` if `x` and `y` are not equal, `1` otherwise.
    Neq,
    /// Returns the second input if the first input is `1`, otherwise
    /// returns the third input.
    IfThenElse,
    /// Returns `x` % `y`
    Modulo,
}

impl Op {
    /// Processes a [`GraphProcessorItem`] for [`Op`] nodes.
    pub fn run<'a>(
        item: &'a mut GraphProcessorItem<'a, Op, OpNum>,
    ) -> ::std::result::Result<(), OpError> {
        let values = item.inputs();
        let v: Vec<OpNum> = values
            .iter()
            .filter_map(|(_, value)| value.as_ref().cloned())
            .collect();
        let inner = item.node().inner().ok_or(OpError::MissingInner)?;
        let output = match inner {
            Op::Add => v[0] + v[1],
            Op::Subtract => v[0] - v[1],
            Op::Multiply => v[0] * v[1],
            Op::Divide => v[0] / v[1],
            Op::Identity(val) => *val,
            Op::Max => v
                .into_iter()
                .reduce(OpNum::max)
                .ok_or(OpError::InvalidArgument)?,
            Op::Min => v
                .into_iter()
                .reduce(OpNum::min)
                .ok_or(OpError::InvalidArgument)?,
            Op::Eq => {
                if v[0] == v[1] {
                    1.0
                } else {
                    0.0
                }
            }
            Op::Neq => {
                if v[0] != v[1] {
                    1.0
                } else {
                    0.0
                }
            }
            Op::IfThenElse => {
                if v[0] == 1.0 {
                    v[1]
                } else {
                    v[2]
                }
            }
            Op::Modulo => v[0] % v[1],
        };

        {
            let out_value = item
                .outputs_mut()
                .iter_mut()
                .find_map(|(key, value)| if *key == "out" { Some(value) } else { None })
                .ok_or(OpError::MissingOutput)?;
            **out_value = Some(output);
        }
        Ok(())
    }
}

#[cfg(feature = "render")]
impl crate::RenderableValue for OpNum {
    fn render_value(&self) -> String {
        self.to_string()
    }
}

impl From<CommonGraphError> for OpError {
    fn from(value: CommonGraphError) -> Self {
        OpError::Other(value.to_string())
    }
}

impl From<OpError> for CommonGraphError {
    fn from(value: OpError) -> Self {
        CommonGraphError::InternalError(value.to_string())
    }
}
