use super::SocketResult;
use crate::wasi::bindings::sockets::tcp::ErrorCode;
use crate::wasi::bindings::{
    io::streams::{InputStream, OutputStream},
    sockets::network::{IpAddressFamily, IpSocketAddress, Network},
    sockets::tcp::{self, ShutdownType},
};
use crate::wasi::poll::{Pollable, Subscribe};
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

/// A host TCP socket, plus associated bookkeeping.
pub struct TcpSocket {}

#[async_trait::async_trait]
impl Subscribe for TcpSocket {
    async fn ready(&mut self) {}
}

impl<T> crate::wasi::bindings::sockets::tcp_create_socket::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn create_tcp_socket(
        &mut self,
        _address_family: IpAddressFamily,
    ) -> SocketResult<Resource<TcpSocket>> {
        Err(ErrorCode::NotSupported.into())
    }
}

impl<T> crate::wasi::bindings::sockets::tcp::Host for WasiImpl<T> where T: WasiView {}

#[async_trait::async_trait]
impl<T> crate::wasi::bindings::sockets::tcp::HostTcpSocket for WasiImpl<T>
where
    T: WasiView,
{
    async fn start_bind(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _network: Resource<Network>,
        _local_address: IpSocketAddress,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn finish_bind(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    async fn start_connect(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _network: Resource<Network>,
        _remote_address: IpSocketAddress,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn finish_connect(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
    ) -> SocketResult<(Resource<InputStream>, Resource<OutputStream>)> {
        Err(ErrorCode::NotSupported.into())
    }

    fn start_listen(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn finish_listen(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn accept(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
    ) -> SocketResult<(
        Resource<tcp::TcpSocket>,
        Resource<InputStream>,
        Resource<OutputStream>,
    )> {
        Err(ErrorCode::NotSupported.into())
    }

    fn local_address(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<IpSocketAddress> {
        Err(ErrorCode::NotSupported.into())
    }

    fn remote_address(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<IpSocketAddress> {
        Err(ErrorCode::NotSupported.into())
    }

    fn is_listening(&mut self, _this: Resource<tcp::TcpSocket>) -> Result<bool, anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }

    fn address_family(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
    ) -> Result<IpAddressFamily, anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_listen_backlog_size(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn keep_alive_enabled(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<bool> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_keep_alive_enabled(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: bool,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn keep_alive_idle_time(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_keep_alive_idle_time(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn keep_alive_interval(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_keep_alive_interval(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn keep_alive_count(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u32> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_keep_alive_count(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u32,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn hop_limit(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u8> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_hop_limit(&mut self, _this: Resource<tcp::TcpSocket>, _value: u8) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn receive_buffer_size(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_receive_buffer_size(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn send_buffer_size(&mut self, _this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        Err(ErrorCode::NotSupported.into())
    }

    fn set_send_buffer_size(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _value: u64,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn subscribe(&mut self, _this: Resource<tcp::TcpSocket>) -> anyhow::Result<Resource<Pollable>> {
        Err(ErrorCode::NotSupported.into())
    }

    fn shutdown(
        &mut self,
        _this: Resource<tcp::TcpSocket>,
        _shutdown_type: ShutdownType,
    ) -> SocketResult<()> {
        Err(ErrorCode::NotSupported.into())
    }

    fn drop(&mut self, _this: Resource<tcp::TcpSocket>) -> Result<(), anyhow::Error> {
        Err(ErrorCode::NotSupported.into())
    }
}

pub mod sync {
    use super::super::SocketError;
    use crate::wasi::bindings::{
        sockets::network::{ErrorCode, Network},
        sync::sockets::tcp::{
            self, Duration, HostTcpSocket, InputStream, IpAddressFamily, IpSocketAddress,
            OutputStream, Pollable, ShutdownType, TcpSocket,
        },
    };
    use crate::wasi::{WasiImpl, WasiView};
    use wasmtime::component::Resource;

    impl<T> tcp::Host for WasiImpl<T> where T: WasiView {}

    impl<T> HostTcpSocket for WasiImpl<T>
    where
        T: WasiView,
    {
        fn start_bind(
            &mut self,
            _this: Resource<TcpSocket>,
            _network: Resource<Network>,
            _local_address: IpSocketAddress,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn finish_bind(&mut self, _this: Resource<TcpSocket>) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn start_connect(
            &mut self,
            _this: Resource<TcpSocket>,
            _network: Resource<Network>,
            _remote_address: IpSocketAddress,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn finish_connect(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<(Resource<InputStream>, Resource<OutputStream>), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn start_listen(&mut self, _this: Resource<TcpSocket>) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn finish_listen(&mut self, _this: Resource<TcpSocket>) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn accept(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<
            (
                Resource<TcpSocket>,
                Resource<InputStream>,
                Resource<OutputStream>,
            ),
            SocketError,
        > {
            Err(ErrorCode::NotSupported.into())
        }

        fn local_address(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<IpSocketAddress, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn remote_address(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<IpSocketAddress, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn is_listening(&mut self, _this: Resource<TcpSocket>) -> wasmtime::Result<bool> {
            Err(ErrorCode::NotSupported.into())
        }

        fn address_family(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> wasmtime::Result<IpAddressFamily> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_listen_backlog_size(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: u64,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn keep_alive_enabled(&mut self, _this: Resource<TcpSocket>) -> Result<bool, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_keep_alive_enabled(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: bool,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn keep_alive_idle_time(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<Duration, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_keep_alive_idle_time(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: Duration,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn keep_alive_interval(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> Result<Duration, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_keep_alive_interval(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: Duration,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn keep_alive_count(&mut self, _this: Resource<TcpSocket>) -> Result<u32, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_keep_alive_count(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: u32,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn hop_limit(&mut self, _this: Resource<TcpSocket>) -> Result<u8, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_hop_limit(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: u8,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn receive_buffer_size(&mut self, _this: Resource<TcpSocket>) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_receive_buffer_size(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: u64,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn send_buffer_size(&mut self, _this: Resource<TcpSocket>) -> Result<u64, SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn set_send_buffer_size(
            &mut self,
            _this: Resource<TcpSocket>,
            _value: u64,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn subscribe(
            &mut self,
            _this: Resource<TcpSocket>,
        ) -> wasmtime::Result<Resource<Pollable>> {
            Err(ErrorCode::NotSupported.into())
        }

        fn shutdown(
            &mut self,
            _this: Resource<TcpSocket>,
            _shutdown_type: ShutdownType,
        ) -> Result<(), SocketError> {
            Err(ErrorCode::NotSupported.into())
        }

        fn drop(&mut self, _rep: Resource<TcpSocket>) -> wasmtime::Result<()> {
            Err(ErrorCode::NotSupported.into())
        }
    }
}
