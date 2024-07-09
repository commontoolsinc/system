/// Synchronous-generated bindings for WASI interfaces.
pub mod sync {
    mod generated {
        use crate::wasi::StreamError;

        wasmtime::component::bindgen!({
            path: "wit",
            world: "common:wasi/imports",
            tracing: true,
            trappable_error_type: {
                "wasi:io/streams/stream-error" => StreamError,
                "wasi:filesystem/types/error-code" => crate::wasi::stubs::filesystem::FsError,
                "wasi:sockets/network/error-code" => crate::wasi::stubs::sockets::SocketError,
                "wasi:http/types/error-code" => crate::wasi::stubs::http::HttpError,
            },
            trappable_imports: true,
            with: {
                // These interfaces come from the outer module, as it's
                // sync/async agnostic.
                "wasi:clocks": crate::wasi::bindings::clocks,
                "wasi:random": crate::wasi::bindings::random,
                "wasi:cli": crate::wasi::bindings::cli,
                "wasi:io/error": crate::wasi::bindings::io::error,
                "wasi:filesystem/preopens": crate::wasi::bindings::filesystem::preopens,
                "wasi:sockets/network": crate::wasi::bindings::sockets::network,
                "wasi:http": crate::wasi::bindings::http,

                // Configure the resource types of the bound interfaces here
                // to be the same as the async versions of the resources, that
                // way everything has the same type.
                "wasi:io/poll/pollable": super::super::io::poll::Pollable,
                "wasi:io/streams/input-stream": super::super::io::streams::InputStream,
                "wasi:io/streams/output-stream": super::super::io::streams::OutputStream,

                // Stubbed interfaces
                "wasi:filesystem/types/descriptor": super::super::filesystem::types::Descriptor,
                "wasi:filesystem/types/directory-entry-stream": super::super::filesystem::types::DirectoryEntryStream,
                "wasi:sockets/tcp/tcp-socket": crate::wasi::stubs::sockets::TcpSocket,
                "wasi:sockets/udp/incoming-datagram-stream": crate::wasi::stubs::sockets::IncomingDatagramStream,
                "wasi:sockets/udp/outgoing-datagram-stream": crate::wasi::stubs::sockets::OutgoingDatagramStream,
                "wasi:sockets/udp/udp-socket": crate::wasi::stubs::sockets::UdpSocket,
            },
            require_store_data_send: true,
        });
    }
    pub use self::generated::exports;
    pub use self::generated::wasi::*;

    // `Command` not currently exported.
    //pub use self::generated::Command;
    //pub use self::generated::CommandPre;
}

mod async_io {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "common:wasi/imports",
        tracing: true,
        trappable_imports: true,
        async: {
            // Only these functions are `async` and everything else is sync
            // meaning that it basically doesn't need to block. These functions
            // are the only ones that need to block.
            //
            // Note that at this time `only_imports` works on function names
            // which in theory can be shared across interfaces, so this may
            // need fancier syntax in the future.
            only_imports: [
                "[method]descriptor.access-at",
                "[method]descriptor.advise",
                "[method]descriptor.change-directory-permissions-at",
                "[method]descriptor.change-file-permissions-at",
                "[method]descriptor.create-directory-at",
                "[method]descriptor.get-flags",
                "[method]descriptor.get-type",
                "[method]descriptor.is-same-object",
                "[method]descriptor.link-at",
                "[method]descriptor.lock-exclusive",
                "[method]descriptor.lock-shared",
                "[method]descriptor.metadata-hash",
                "[method]descriptor.metadata-hash-at",
                "[method]descriptor.open-at",
                "[method]descriptor.read",
                "[method]descriptor.read-directory",
                "[method]descriptor.readlink-at",
                "[method]descriptor.remove-directory-at",
                "[method]descriptor.rename-at",
                "[method]descriptor.set-size",
                "[method]descriptor.set-times",
                "[method]descriptor.set-times-at",
                "[method]descriptor.stat",
                "[method]descriptor.stat-at",
                "[method]descriptor.symlink-at",
                "[method]descriptor.sync",
                "[method]descriptor.sync-data",
                "[method]descriptor.try-lock-exclusive",
                "[method]descriptor.try-lock-shared",
                "[method]descriptor.unlink-file-at",
                "[method]descriptor.unlock",
                "[method]descriptor.write",
                "[method]input-stream.read",
                "[method]input-stream.blocking-read",
                "[method]input-stream.blocking-skip",
                "[method]input-stream.skip",
                "[method]output-stream.forward",
                "[method]output-stream.splice",
                "[method]output-stream.blocking-splice",
                "[method]output-stream.blocking-flush",
                "[method]output-stream.blocking-write",
                "[method]output-stream.blocking-write-and-flush",
                "[method]output-stream.blocking-write-zeroes-and-flush",
                "[method]directory-entry-stream.read-directory-entry",
                "poll",
                "[method]pollable.block",
                "[method]pollable.ready",
                "[method]tcp-socket.start-bind",
                "[method]tcp-socket.start-connect",
                "[method]udp-socket.start-bind",
                "[method]udp-socket.stream",
                "[method]outgoing-datagram-stream.send",
            ],
        },
        trappable_error_type: {
            "wasi:io/streams/stream-error" => crate::wasi::StreamError,
            "wasi:filesystem/types/error-code" => crate::wasi::stubs::filesystem::FsError,
            "wasi:sockets/network/error-code" => crate::wasi::stubs::sockets::SocketError,
            "wasi:http/types/error-code" => crate::wasi::stubs::http::HttpError,
        },
        with: {
            // Configure all resources to be concrete types defined in this crate,
            // so that way we get to use nice typed helper methods with
            // `ResourceTable`.

            "wasi:io/streams/input-stream": crate::wasi::stream::InputStream,
            "wasi:io/streams/output-stream": crate::wasi::stream::OutputStream,
            "wasi:io/error/error": crate::wasi::stream::Error,
            "wasi:io/poll/pollable": crate::wasi::poll::Pollable,
            "wasi:cli/terminal-input/terminal-input": crate::wasi::stdio::TerminalInput,
            "wasi:cli/terminal-output/terminal-output": crate::wasi::stdio::TerminalOutput,

            // Stubbed interfaces
            "wasi:filesystem/types/descriptor": crate::wasi::stubs::filesystem::Descriptor,
            "wasi:filesystem/types/directory-entry-stream": crate::wasi::stubs::filesystem::ReaddirIterator,
            "wasi:http/types/outgoing-body": crate::wasi::stubs::http::HostOutgoingBody,
            "wasi:http/types/future-incoming-response": crate::wasi::stubs::http::HostFutureIncomingResponse,
            "wasi:http/types/outgoing-response": crate::wasi::stubs::http::HostOutgoingResponse,
            "wasi:http/types/future-trailers": crate::wasi::stubs::http::HostFutureTrailers,
            "wasi:http/types/incoming-body": crate::wasi::stubs::http::HostIncomingBody,
            "wasi:http/types/incoming-response": crate::wasi::stubs::http::HostIncomingResponse,
            "wasi:http/types/response-outparam": crate::wasi::stubs::http::HostResponseOutparam,
            "wasi:http/types/outgoing-request": crate::wasi::stubs::http::HostOutgoingRequest,
            "wasi:http/types/incoming-request": crate::wasi::stubs::http::HostIncomingRequest,
            "wasi:http/types/fields": crate::wasi::stubs::http::HostFields,
            "wasi:http/types/request-options": crate::wasi::stubs::http::HostRequestOptions,
            "wasi:sockets/ip-name-lookup/resolve-address-stream": crate::wasi::stubs::sockets::ResolveAddressStream,
            "wasi:sockets/network/network": crate::wasi::stubs::sockets::Network,
            "wasi:sockets/tcp/tcp-socket": crate::wasi::stubs::sockets::TcpSocket,
            "wasi:sockets/udp/udp-socket": crate::wasi::stubs::sockets::UdpSocket,
            "wasi:sockets/udp/incoming-datagram-stream": crate::wasi::stubs::sockets::IncomingDatagramStream,
            "wasi:sockets/udp/outgoing-datagram-stream": crate::wasi::stubs::sockets::OutgoingDatagramStream,
        },
    });
}

pub use self::async_io::exports;
pub use self::async_io::wasi::*;

// `Command` not currently exported.
//pub use self::async_io::Command;
//pub use self::async_io::CommandPre;
