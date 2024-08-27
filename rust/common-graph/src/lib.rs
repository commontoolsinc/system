#![warn(missing_docs)]

//! Port graph functionality for Common runtime.
//!
//! [`Graph`] represents a [directed acyclic graph] consisting
//! of nodes with named ports, where output ports can connect to
//! other nodes' input ports. The [`Graph`] itself also has
//! its own inputs and outputs into the graph, and provides
//! a [`Graph::process`] function that takes input data,
//! and propagates data through the network, generating output.
//!
//! # Example
//!
//! This example uses `Op` nodes which represent some math operation,
//! only supporting the `Op::Square` function, which squares its
//! inputs, and writes the result for each input to an output.
//!
//! A [`Graph`] is constructed via [`GraphBuilder`] to square inputs.
//! Calling [`Graph::process`] starts iterating
//! over nodes, calling the provided callback for each node,
//! where input data is processed, and output ports are written to.
//!
//! After processing is completed, the full result of data contained
//! in all ports is returned.
//!
//! ```
//! use common_graph::{GraphBuilder, GraphProcessorItem, Result};
//! # fn main() -> Result<()> {
//!
//! #[derive(Clone, PartialEq, Debug)]
//! enum Op {
//!     Square,
//! }
//!
//! // Square all inputs.
//! let graph = GraphBuilder::default()
//!     .set_graph_input(["x", "y"])
//!     .set_graph_output(["x", "y"])
//!     .node("square", Op::Square, ["x", "y"], ["x", "y"])
//!     .connect_input("x", ("square", "x"))?
//!     .connect_input("y", ("square", "y"))?
//!     .connect_output(("square", "x"), "x")?
//!     .connect_output(("square", "y"), "y")?
//!     .build()?;
//!
//! let input = [("x", 3), ("y", 5)];
//! let output = graph.process(input, |item: &mut GraphProcessorItem<'_, _, _>| {
//!     let inner = item.node().inner().ok_or(String::from("No inner"))?.clone();
//!     let inputs = item.inputs().to_owned();
//!
//!     for (key, out_value) in item.outputs_mut() {
//!         let (_, in_value) = inputs.iter().find(|(k, _)| k == key)
//!             .ok_or(String::from("No key"))?;
//!         let in_value = in_value.ok_or(String::from("Empty input"))?;
//!         **out_value = match inner {
//!             Op::Square => Some(in_value * in_value),
//!         };
//!     }
//!     Ok::<(), String>(())
//! })??;
//!
//! let root_out = output.into_inner()[0].0.clone();
//! assert_eq!(root_out, vec![("x", Some(9)), ("y", Some(25))]);
//! # Ok(())
//! # }
//! ```
//!
//! [directed acyclic graph]: https://en.wikipedia.org/wiki/Directed_acyclic_graph

mod builder;
mod error;
mod graph;
#[cfg(feature = "helpers")]
pub mod helpers;
mod processor;
#[cfg(feature = "render")]
mod render;
mod storage;
mod utils;

pub use builder::*;
pub use error::*;
pub use graph::*;
pub use processor::*;
#[cfg(feature = "render")]
pub use render::*;
pub use storage::*;
