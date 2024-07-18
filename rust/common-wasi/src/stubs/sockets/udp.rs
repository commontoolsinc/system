use super::SocketResult;
use crate::{
    bindings::{
        sockets::network::{ErrorCode, IpAddressFamily, IpSocketAddress, Network},
        sockets::udp,
    },
    poll::{Pollable, Subscribe},
};
use crate::{WasiImpl, WasiView};
use async_trait::async_trait;
use wasmtime::component::Resource;

/// A host UDP socket, plus associated bookkeeping.
pub struct UdpSocket {}

#[async_trait]
impl Subscribe for UdpSocket {
    async fn ready(&mut self) {}
}

pub struct IncomingDatagramStream {}

pub struct OutgoingDatagramStream {}

impl<T> crate::bindings::sockets::udp_create_socket::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn create_udp_socket(
        &mut self,
        _address_family: IpAddressFamily,
    ) -> SocketResult<Resource<UdpSocket>> {
        Err(ErrorCode::NotSupported.into())
    }
}

impl<T> udp::Host for WasiImpl<T> where T: WasiView {}

#[async_trait::async_trait]
impl<T> udp::HostUdpSocket for WasiImpl<T>
where
    T: WasiView,
{
    async fn start_bind(
        &mut self,
        _this: Resource<udp::UdpSocket>,
        _network: Resource<Network>,
        _local_address: IpSocketAddress,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn finish_bind(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    async fn stream(
        &mut self,
        _this: Resource<udp::UdpSocket>,
        _remote_address: Option<IpSocketAddress>,
    ) -> SocketResult<(
        Resource<udp::IncomingDatagramStream>,
        Resource<udp::OutgoingDatagramStream>,
    )> {
        Err(ErrorCode::NotSupported.into())
    }

    fn local_address(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<IpSocketAddress> {
        Err(ErrorCode::NotSupported.into())
    }

    fn remote_address(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<IpSocketAddress> {
        Err(ErrorCode::NotSupported.into())
    }

    fn address_family(
        &mut self,
        _this: Resource<udp::UdpSocket>,
    ) -> Result<IpAddressFamily, anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }

    fn unicast_hop_limit(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<u8> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_unicast_hop_limit(
        &mut self,
        _this: Resource<udp::UdpSocket>,
        _value: u8,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn receive_buffer_size(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_receive_buffer_size(
        &mut self,
        _this: Resource<udp::UdpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn send_buffer_size(&mut self, _this: Resource<udp::UdpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_send_buffer_size(
        &mut self,
        _this: Resource<udp::UdpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn subscribe(&mut self, _this: Resource<udp::UdpSocket>) -> anyhow::Result<Resource<Pollable>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn drop(&mut self, _this: Resource<udp::UdpSocket>) -> Result<(), anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}

impl<T> udp::HostIncomingDatagramStream for WasiImpl<T>
where
    T: WasiView,
{
    fn receive(
        &mut self,
        _this: Resource<udp::IncomingDatagramStream>,
        _max_results: u64,
    ) -> SocketResult<Vec<udp::IncomingDatagram>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn subscribe(
        &mut self,
        _this: Resource<udp::IncomingDatagramStream>,
    ) -> anyhow::Result<Resource<Pollable>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn drop(&mut self, _this: Resource<udp::IncomingDatagramStream>) -> Result<(), anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}

#[async_trait]
impl Subscribe for IncomingDatagramStream {
    async fn ready(&mut self) {}
}

#[async_trait::async_trait]
impl<T> udp::HostOutgoingDatagramStream for WasiImpl<T>
where
    T: WasiView,
{
    fn check_send(&mut self, _this: Resource<udp::OutgoingDatagramStream>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    async fn send(
        &mut self,
        _this: Resource<udp::OutgoingDatagramStream>,
        _datagrams: Vec<udp::OutgoingDatagram>,
    ) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn subscribe(
        &mut self,
        _this: Resource<udp::OutgoingDatagramStream>,
    ) -> anyhow::Result<Resource<Pollable>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn drop(&mut self, _this: Resource<udp::OutgoingDatagramStream>) -> Result<(), anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}

#[async_trait]
impl Subscribe for OutgoingDatagramStream {
    async fn ready(&mut self) {}
}

pub mod sync {
    use super::super::SocketError;
    use crate::{
        bindings::{
            sockets::{
                network::{ErrorCode, Network},
                udp::{IncomingDatagramStream, OutgoingDatagramStream},
            },
            sync::sockets::udp::{
                self, HostIncomingDatagramStream, HostOutgoingDatagramStream, HostUdpSocket,
                IncomingDatagram, IpAddressFamily, IpSocketAddress, OutgoingDatagram, Pollable,
                UdpSocket,
            },
        },
        WasiImpl, WasiView,
    };
    use wasmtime::component::Resource;

    impl<T> udp::Host for WasiImpl<T> where T: WasiView {}

    impl<T> HostUdpSocket for WasiImpl<T>
    where
        T: WasiView,
    {
        fn start_bind(
            &mut self,
            _this: Resource<UdpSocket>,
            _network: Resource<Network>,
            _local_address: IpSocketAddress,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn finish_bind(&mut self, _this: Resource<UdpSocket>) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn stream(
            &mut self,
            _this: Resource<UdpSocket>,
            _remote_address: Option<IpSocketAddress>,
        ) -> Result<
            (
                Resource<IncomingDatagramStream>,
                Resource<OutgoingDatagramStream>,
            ),
            SocketError,
        > {
            Err(ErrorCode::NotSupported.into())
        }

        fn local_address(
            &mut self,
            _this: Resource<UdpSocket>,
        ) -> Result<IpSocketAddress, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn remote_address(
            &mut self,
            _this: Resource<UdpSocket>,
        ) -> Result<IpSocketAddress, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn address_family(
            &mut self,
            _this: Resource<UdpSocket>,
        ) -> wasmtime::Result<IpAddressFamily> {
            Err(ErrorCode::NotSupported.into())
        }

        fn unicast_hop_limit(&mut self, _this: Resource<UdpSocket>) -> Result<u8, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_unicast_hop_limit(
            &mut self,
            _this: Resource<UdpSocket>,
            _value: u8,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn receive_buffer_size(&mut self, _this: Resource<UdpSocket>) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_receive_buffer_size(
            &mut self,
            _this: Resource<UdpSocket>,
            _value: u64,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn send_buffer_size(&mut self, _this: Resource<UdpSocket>) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_send_buffer_size(
            &mut self,
            _this: Resource<UdpSocket>,
            _value: u64,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn subscribe(
            &mut self,
            _this: Resource<UdpSocket>,
        ) -> wasmtime::Result<Resource<Pollable>> {
            Err(ErrorCode::NotSupported.into())
        }

        fn drop(&mut self, _rep: Resource<UdpSocket>) -> wasmtime::Result<()> {
            Err(ErrorCode::NotSupported.into())
        }
    }

    impl<T> HostIncomingDatagramStream for WasiImpl<T>
    where
        T: WasiView,
    {
        fn receive(
            &mut self,
            _this: Resource<IncomingDatagramStream>,
            _max_results: u64,
        ) -> Result<Vec<IncomingDatagram>, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn subscribe(
            &mut self,
            _this: Resource<IncomingDatagramStream>,
        ) -> wasmtime::Result<Resource<Pollable>> {
            Err(ErrorCode::NotSupported.into())
        }

        fn drop(&mut self, _rep: Resource<IncomingDatagramStream>) -> wasmtime::Result<()> {
            Err(ErrorCode::NotSupported.into())
        }
    }

    impl<T> HostOutgoingDatagramStream for WasiImpl<T>
    where
        T: WasiView,
    {
        fn check_send(
            &mut self,
            _this: Resource<OutgoingDatagramStream>,
        ) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn send(
            &mut self,
            _this: Resource<OutgoingDatagramStream>,
            _datagrams: Vec<OutgoingDatagram>,
        ) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn subscribe(
            &mut self,
            _this: Resource<OutgoingDatagramStream>,
        ) -> wasmtime::Result<Resource<Pollable>> {
            Err(ErrorCode::NotSupported.into())
        }

        fn drop(&mut self, _rep: Resource<OutgoingDatagramStream>) -> wasmtime::Result<()> {
            Err(ErrorCode::NotSupported.into())
        }
    }
}
