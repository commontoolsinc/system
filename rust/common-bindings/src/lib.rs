#[macro_use]
extern crate tracing;

pub mod bindings;
mod common;
mod ctx;
pub mod wasi;

pub use crate::bindings::{
    common_module::Common as CommonModuleEntry, common_script::Common as CommonScriptEntry,
};
pub use common::*;
pub use ctx::{BindingsContext, BindingsContextBuilder, BindingsView};

#[cfg(not(target_arch = "wasm32"))]
pub async fn instantiate_async_module<T: BindingsView>(
    store: &mut impl wasmtime::AsContextMut<Data = T>,
    component: &wasmtime::component::Component,
    linker: &wasmtime::component::Linker<T>,
) -> anyhow::Result<(CommonModuleEntry, wasmtime::component::Instance)> {
    CommonModuleEntry::instantiate_async(store, component, linker).await
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn instantiate_async_script<T: BindingsView>(
    store: &mut impl wasmtime::AsContextMut<Data = T>,
    component: &wasmtime::component::Component,
    linker: &wasmtime::component::Linker<T>,
) -> anyhow::Result<(CommonScriptEntry, wasmtime::component::Instance)> {
    CommonScriptEntry::instantiate_async(store, component, linker).await
}

#[cfg(not(target_arch = "wasm32"))]
pub fn link_common_module<T: BindingsView>(
    linker: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    fn type_annotate<T: BindingsView, F>(val: F) -> F
    where
        F: Fn(&mut T) -> crate::ctx::BindingsImpl<&mut T>,
    {
        val
    }

    let closure = type_annotate::<T, _>(|t| crate::ctx::BindingsImpl(t));

    use crate::bindings::common_module::common;
    common::data::types::add_to_linker_get_host(linker, closure)?;
    common::io::state::add_to_linker_get_host(linker, closure)?;
    common::module::reflect::add_to_linker_get_host(linker, closure)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn link_common_script<T: BindingsView>(
    linker: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    fn type_annotate<T: BindingsView, F>(val: F) -> F
    where
        F: Fn(&mut T) -> crate::ctx::BindingsImpl<&mut T>,
    {
        val
    }

    let closure = type_annotate::<T, _>(|t| crate::ctx::BindingsImpl(t));

    use crate::bindings::common_script::common;
    common::data::types::add_to_linker_get_host(linker, closure)?;
    common::io::state::add_to_linker_get_host(linker, closure)?;
    common::module::reflect::add_to_linker_get_host(linker, closure)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn link_wasi_sync<T: crate::wasi::WasiView>(
    linker: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    crate::wasi::add_to_linker_sync(linker)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn link_wasi_async<T: crate::wasi::WasiView>(
    linker: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()> {
    crate::wasi::add_to_linker_async(linker)
}
