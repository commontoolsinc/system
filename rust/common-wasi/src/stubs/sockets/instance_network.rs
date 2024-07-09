use super::Network;
use crate::bindings::sockets::network::ErrorCode;
use crate::{WasiImpl, WasiView};
use anyhow::Result;
use wasmtime::component::Resource;

impl<T> crate::bindings::sockets::instance_network::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn instance_network(&mut self) -> Result<Resource<Network>, anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}
