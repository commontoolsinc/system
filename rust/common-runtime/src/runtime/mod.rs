#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod web;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use web::*;

mod io;
pub use io::*;
