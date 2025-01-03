#![warn(missing_docs)]
//! `ct-storage` is a passive database, currently a thin wrapper
//! around `ranked-prolly-tree`.

mod ct_storage;
mod encoding;
mod error;
mod key;
mod storage;

pub use ct_storage::*;
pub use error::*;
pub use key::*;
pub use storage::*;
