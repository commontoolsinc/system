mod context;
mod traits;

pub use traits::*;

#[cfg(target_arch = "wasm32")]
mod wcl;
#[cfg(target_arch = "wasm32")]
pub use wcl::{WclEngine as Engine, WclInstance as Instance, WclModule as Module};

#[cfg(not(target_arch = "wasm32"))]
mod wasmtime;
#[cfg(not(target_arch = "wasm32"))]
pub use wasmtime::{
    WasmtimeEngine as Engine, WasmtimeInstance as Instance, WasmtimeModule as Module,
};
