use crate::wasi::bindings::sockets::network::ErrorCode;
use crate::wasi::error::TrappableError;
use anyhow::Result;

mod instance_network;
mod ip_name_lookup;
mod network;
mod tcp;
mod udp;

pub use ip_name_lookup::*;
pub use network::*;
pub use tcp::*;
pub use udp::*;

pub type SocketResult<T> = Result<T, SocketError>;
pub type SocketError = TrappableError<ErrorCode>;
