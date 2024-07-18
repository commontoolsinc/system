use super::{Network, SocketError};
use crate::bindings::sockets::network::{ErrorCode, IpAddress};
use crate::poll::{Pollable, Subscribe};
use crate::{WasiImpl, WasiView};
use anyhow::Result;
use wasmtime::component::Resource;

pub enum ResolveAddressStream {
    Waiting,
    Done,
}

#[async_trait::async_trait]
impl Subscribe for ResolveAddressStream {
    async fn ready(&mut self) {}
}

impl<T> crate::bindings::sockets::ip_name_lookup::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn resolve_addresses(
        &mut self,
        _network: Resource<Network>,
        _name: String,
    ) -> Result<Resource<ResolveAddressStream>, SocketError> {
        Err(ErrorCode::NotSupported.into())
    }
}

#[async_trait::async_trait]
impl<T> crate::bindings::sockets::ip_name_lookup::HostResolveAddressStream for WasiImpl<T>
where
    T: WasiView,
{
    fn resolve_next_address(
        &mut self,
        _resource: Resource<ResolveAddressStream>,
    ) -> Result<Option<IpAddress>, SocketError> {
        Err(ErrorCode::NotSupported.into())
    }

    fn subscribe(
        &mut self,
        _resource: Resource<ResolveAddressStream>,
    ) -> Result<Resource<Pollable>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn drop(&mut self, _resource: Resource<ResolveAddressStream>) -> Result<()> {
        Err(ErrorCode::NotSupported.into())
    }
}
