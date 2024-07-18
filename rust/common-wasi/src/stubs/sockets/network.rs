use super::SocketError;
use crate::bindings::sockets::network::ErrorCode;
use crate::{WasiImpl, WasiView};
use anyhow::Result;
use wasmtime::component::Resource;

pub struct Network {}

impl<T> crate::bindings::sockets::network::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn convert_error_code(&mut self, _error: SocketError) -> anyhow::Result<ErrorCode> {
        Err(ErrorCode::NotSupported.into())
    }
}

impl<T> crate::bindings::sockets::network::HostNetwork for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _this: Resource<Network>) -> Result<(), anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}
