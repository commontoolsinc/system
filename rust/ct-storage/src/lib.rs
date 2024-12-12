#![warn(missing_docs)]
//! `ct-storage` is a passive database, currently a thin wrapper
//! around `ranked-prolly-tree`.

pub use ranked_prolly_tree::Error;

mod platform;
mod storage;

pub use storage::*;
