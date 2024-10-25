mod context;

#[cfg(target_arch = "wasm32")]
mod wcl;
#[cfg(target_arch = "wasm32")]
pub use wcl::*;

#[cfg(not(target_arch = "wasm32"))]
mod wasmtime;
#[cfg(not(target_arch = "wasm32"))]
pub use wasmtime::*;
