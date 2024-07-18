//! # Wasmtime's WASI Implementation
//!
//! This crate provides a Wasmtime host implementation of WASI 0.2 (aka WASIp2
//! aka Preview 2), originally forked from [wasmtime-wasi](https://docs.rs/wasmtime-wasi)
//! and [wasmtime-wasi-http](https://docs.rs/wasmtime-wasi-http).
//!
//! Deviations from the original project are as follows:
//!
//! * Some interfaces are stubbed. Stubbed interfaces provide resources and
//!   functions that may be executed, but returns an unsupported error.
//!   See the list below of interface support.
//! * Updated `WasiCtxBuilder` with capabilities within the context of
//!   Common Modules.
//!   * Remove features related to stubbed interfaces.
//!   * Cannot inherit streams, environment, or arguments from the host.
//! * Remove WASI Preview 0 and Preview 1 support.
//!
//! # WASIp2 interfaces
//!
//! This crate contains implementations of the following interfaces,
//! either supported or stubbed:
//!
//! ## Supported interfaces
//!
//! * [`wasi:cli/environment`]
//! * [`wasi:cli/exit`]
//! * [`wasi:cli/stderr`]
//! * [`wasi:cli/stdin`]
//! * [`wasi:cli/stdout`]
//! * [`wasi:cli/terminal-input`]
//! * [`wasi:cli/terminal-output`]
//! * [`wasi:cli/terminal-stderr`]
//! * [`wasi:cli/terminal-stdin`]
//! * [`wasi:cli/terminal-stdout`]
//! * [`wasi:clocks/monotonic-clock`]
//! * [`wasi:clocks/wall-clock`]
//! * [`wasi:io/error`]
//! * [`wasi:io/poll`]
//! * [`wasi:io/streams`]
//! * [`wasi:random/insecure-seed`]
//! * [`wasi:random/insecure`]
//! * [`wasi:random/random`]
//!
//! ## Stubbed interfaces
//!
//! * [`wasi:filesystem/preopens`]
//! * [`wasi:filesystem/types`]
//! * [`wasi:http/outgoing-handler`]
//! * [`wasi:http/types`]
//! * [`wasi:http/incoming-handler`]
//! * [`wasi:http/proxy`]
//! * [`wasi:sockets/instance-network`]
//! * [`wasi:sockets/ip-name-lookup`]
//! * [`wasi:sockets/network`]
//! * [`wasi:sockets/tcp-create-socket`]
//! * [`wasi:sockets/tcp`]
//! * [`wasi:sockets/udp-create-socket`]
//! * [`wasi:sockets/udp`]
//!
//! [`wasi:cli/environment`]: bindings::cli::environment::Host
//! [`wasi:cli/exit`]: bindings::cli::exit::Host
//! [`wasi:cli/stderr`]: bindings::cli::stderr::Host
//! [`wasi:cli/stdin`]: bindings::cli::stdin::Host
//! [`wasi:cli/stdout`]: bindings::cli::stdout::Host
//! [`wasi:cli/terminal-input`]: bindings::cli::terminal_input::Host
//! [`wasi:cli/terminal-output`]: bindings::cli::terminal_output::Host
//! [`wasi:cli/terminal-stdin`]: bindings::cli::terminal_stdin::Host
//! [`wasi:cli/terminal-stdout`]: bindings::cli::terminal_stdout::Host
//! [`wasi:cli/terminal-stderr`]: bindings::cli::terminal_stderr::Host
//! [`wasi:clocks/monotonic-clock`]: bindings::clocks::monotonic_clock::Host
//! [`wasi:clocks/wall-clock`]: bindings::clocks::wall_clock::Host
//! [`wasi:filesystem/preopens`]: bindings::filesystem::preopens::Host
//! [`wasi:filesystem/types`]: bindings::filesystem::types::Host
//! [`wasi:io/error`]: bindings::io::error::Host
//! [`wasi:io/poll`]: bindings::io::poll::Host
//! [`wasi:io/streams`]: bindings::io::streams::Host
//! [`wasi:random/insecure-seed`]: bindings::random::insecure_seed::Host
//! [`wasi:random/insecure`]: bindings::random::insecure::Host
//! [`wasi:random/random`]: bindings::random::random::Host
//! [`wasi:http/outgoing-handler`]: crate::bindings::http::outgoing_handler::Host
//! [`wasi:http/types`]: crate::bindings::http::types::Host
//! [`wasi:http/incoming-handler`]: crate::bindings::exports::wasi::http::incoming_handler::Guest
//! [`wasi:http/proxy`]: crate::bindings::Proxy
//! [`wasi:sockets/instance-network`]: bindings::sockets::instance_network::Host
//! [`wasi:sockets/ip-name-lookup`]: bindings::sockets::ip_name_lookup::Host
//! [`wasi:sockets/network`]: bindings::sockets::network::Host
//! [`wasi:sockets/tcp-create-socket`]: bindings::sockets::tcp_create_socket::Host
//! [`wasi:sockets/tcp`]: bindings::sockets::tcp::Host
//! [`wasi:sockets/udp-create-socket`]: bindings::sockets::udp_create_socket::Host
//! [`wasi:sockets/udp`]: bindings::sockets::udp::Host

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use wasmtime::component::Linker;

pub mod bindings;
mod clocks;
mod ctx;
mod error;
mod host;
pub mod pipe;
mod poll;
mod random;
pub mod runtime;
mod stdio;
mod stream;
mod stubs;
mod write_stream;

pub use self::clocks::{HostMonotonicClock, HostWallClock};
pub use self::ctx::{WasiCtx, WasiCtxBuilder, WasiImpl, WasiView};
pub use self::error::{I32Exit, TrappableError};
pub use self::poll::{subscribe, ClosureFuture, MakeFuture, Pollable, PollableFuture, Subscribe};
pub use self::random::{thread_rng, Deterministic};
pub use self::stdio::{StdinStream, StdoutStream};
pub use self::stream::{
    HostInputStream, HostOutputStream, InputStream, OutputStream, StreamError, StreamResult,
};

#[doc(no_inline)]
pub use async_trait::async_trait;
#[doc(no_inline)]
pub use cap_rand::RngCore;
#[doc(no_inline)]
pub use wasmtime::component::{ResourceTable, ResourceTableError};

/// Add all WASI interfaces from this crate into the `linker` provided.
///
/// This function will add the `async` variant of all interfaces into the
/// [`Linker`] provided. By `async` this means that this function is only
/// compatible with [`Config::async_support(true)`][async]. For embeddings with
/// async support disabled see [`add_to_linker_sync`] instead.
///
/// This function will add all interfaces implemented by this crate to the
/// [`Linker`], which corresponds to the `wasi:cli/imports` world supported by
/// this crate.
///
/// [async]: wasmtime::Config::async_support
///
/// # Example
///
/// ```
/// use wasmtime::{Engine, Result, Store, Config};
/// use wasmtime::component::{ResourceTable, Linker};
/// use common_wasi::{WasiCtx, WasiView, WasiCtxBuilder};
///
/// fn main() -> Result<()> {
///     let mut config = Config::new();
///     config.async_support(true);
///     let engine = Engine::new(&config)?;
///
///     let mut linker = Linker::<MyState>::new(&engine);
///     common_wasi::add_to_linker_async(&mut linker)?;
///     // ... add any further functionality to `linker` if desired ...
///
///     let mut builder = WasiCtxBuilder::new();
///
///     // ... configure `builder` more to add env vars, args, etc ...
///
///     let mut store = Store::new(
///         &engine,
///         MyState {
///             ctx: builder.build(),
///             table: ResourceTable::new(),
///         },
///     );
///
///     // ... use `linker` to instantiate within `store` ...
///
///     Ok(())
/// }
///
/// struct MyState {
///     ctx: WasiCtx,
///     table: ResourceTable,
/// }
///
/// impl WasiView for MyState {
///     fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
///     fn table(&mut self) -> &mut ResourceTable { &mut self.table }
/// }
/// ```
pub fn add_to_linker_async<T: WasiView>(linker: &mut Linker<T>) -> anyhow::Result<()> {
    let l = linker;
    let closure = type_annotate::<T, _>(|t| WasiImpl(t));

    crate::bindings::clocks::wall_clock::add_to_linker_get_host(l, closure)?;
    crate::bindings::clocks::monotonic_clock::add_to_linker_get_host(l, closure)?;
    crate::bindings::filesystem::types::add_to_linker_get_host(l, closure)?;
    crate::bindings::filesystem::preopens::add_to_linker_get_host(l, closure)?;
    crate::bindings::io::error::add_to_linker_get_host(l, closure)?;
    crate::bindings::io::poll::add_to_linker_get_host(l, closure)?;
    crate::bindings::io::streams::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::random::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::insecure::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::insecure_seed::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::exit::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::environment::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stdin::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stdout::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stderr::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_input::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_output::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stdin::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stdout::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stderr::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::tcp::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::tcp_create_socket::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::udp::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::udp_create_socket::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::instance_network::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::network::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::ip_name_lookup::add_to_linker_get_host(l, closure)?;
    crate::bindings::http::outgoing_handler::add_to_linker_get_host(l, closure)?;
    crate::bindings::http::types::add_to_linker_get_host(l, closure)?;
    Ok(())
}

/// Add all WASI interfaces from this crate into the `linker` provided.
///
/// This function will add the synchronous variant of all interfaces into the
/// [`Linker`] provided. By synchronous this means that this function is only
/// compatible with [`Config::async_support(false)`][async]. For embeddings
/// with async support enabled see [`add_to_linker_async`] instead.
///
/// This function will add all interfaces implemented by this crate to the
/// [`Linker`], which corresponds to the `wasi:cli/imports` world supported by
/// this crate.
///
/// [async]: wasmtime::Config::async_support
///
/// # Example
///
/// ```
/// use wasmtime::{Engine, Result, Store, Config};
/// use wasmtime::component::{ResourceTable, Linker};
/// use common_wasi::{WasiCtx, WasiView, WasiCtxBuilder};
///
/// fn main() -> Result<()> {
///     let engine = Engine::default();
///
///     let mut linker = Linker::<MyState>::new(&engine);
///     common_wasi::add_to_linker_sync(&mut linker)?;
///     // ... add any further functionality to `linker` if desired ...
///
///     let mut builder = WasiCtxBuilder::new();
///
///     // ... configure `builder` more to add env vars, args, etc ...
///
///     let mut store = Store::new(
///         &engine,
///         MyState {
///             ctx: builder.build(),
///             table: ResourceTable::new(),
///         },
///     );
///
///     // ... use `linker` to instantiate within `store` ...
///
///     Ok(())
/// }
///
/// struct MyState {
///     ctx: WasiCtx,
///     table: ResourceTable,
/// }
///
/// impl WasiView for MyState {
///     fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
///     fn table(&mut self) -> &mut ResourceTable { &mut self.table }
/// }
/// ```
pub fn add_to_linker_sync<T: WasiView>(
    linker: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    let l = linker;
    let closure = type_annotate::<T, _>(|t| WasiImpl(t));

    crate::bindings::clocks::wall_clock::add_to_linker_get_host(l, closure)?;
    crate::bindings::clocks::monotonic_clock::add_to_linker_get_host(l, closure)?;
    crate::bindings::sync::filesystem::types::add_to_linker_get_host(l, closure)?;
    crate::bindings::filesystem::preopens::add_to_linker_get_host(l, closure)?;
    crate::bindings::io::error::add_to_linker_get_host(l, closure)?;
    crate::bindings::sync::io::poll::add_to_linker_get_host(l, closure)?;
    crate::bindings::sync::io::streams::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::random::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::insecure::add_to_linker_get_host(l, closure)?;
    crate::bindings::random::insecure_seed::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::exit::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::environment::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stdin::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stdout::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::stderr::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_input::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_output::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stdin::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stdout::add_to_linker_get_host(l, closure)?;
    crate::bindings::cli::terminal_stderr::add_to_linker_get_host(l, closure)?;
    crate::bindings::sync::sockets::tcp::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::tcp_create_socket::add_to_linker_get_host(l, closure)?;
    crate::bindings::sync::sockets::udp::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::udp_create_socket::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::instance_network::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::network::add_to_linker_get_host(l, closure)?;
    crate::bindings::sockets::ip_name_lookup::add_to_linker_get_host(l, closure)?;
    crate::bindings::http::outgoing_handler::add_to_linker_get_host(l, closure)?;
    crate::bindings::http::types::add_to_linker_get_host(l, closure)?;
    Ok(())
}

// NB: workaround some rustc inference - a future refactoring may make this
// obsolete.
fn type_annotate<T: WasiView, F>(val: F) -> F
where
    F: Fn(&mut T) -> WasiImpl<&mut T>,
{
    val
}
