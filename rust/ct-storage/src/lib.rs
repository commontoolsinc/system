#![warn(missing_docs)]
//! `ct-storage` is a passive database, currently a thin wrapper
//! around `ranked-prolly-tree`.

mod encoding;
mod error;
mod key;
mod platform;
mod storage;

pub use error::*;
pub use key::*;
pub use platform::PlatformStorage;
pub use storage::*;
