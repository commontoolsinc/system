#[cfg(not(target_arch = "wasm32"))]
pub mod wasmtime;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod browser;
