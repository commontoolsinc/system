use crate::{
    common::bindings::common_module::common::data::types::Reference, wasi::WasiCtx, HostReference,
};
use anyhow::Result;
use common_wit::{ConditionalSend, InputOutput};
use wasmtime::component::{Resource, ResourceTable};

pub trait BindingsView: ConditionalSend {
    type Io: InputOutput;

    fn ctx(&self) -> &BindingsContext<Self::Io>;

    fn ctx_mut(&mut self) -> &mut BindingsContext<Self::Io>;

    fn guest_reference_to_host_reference(
        &self,
        reference: Resource<Reference>,
    ) -> Result<&HostReference, String> {
        let host_resource = Resource::<HostReference>::new_own(reference.rep());
        self.ctx()
            .common_table
            .get(&host_resource)
            .map_err(|error| format!("{error}"))
    }
}

pub struct BindingsContext<Io: InputOutput> {
    pub io: Io,
    #[cfg(not(target_arch = "wasm32"))]
    pub wasi_table: ResourceTable,
    #[cfg(not(target_arch = "wasm32"))]
    pub wasi_ctx: WasiCtx,
    #[cfg(not(target_arch = "wasm32"))]
    pub common_table: ResourceTable,
}

pub struct BindingsContextBuilder<Io: InputOutput> {
    io: Io,
}

impl<Io: InputOutput> BindingsContextBuilder<Io> {
    pub fn new(io: Io) -> Self {
        Self { io }
    }

    pub fn build(self) -> BindingsContext<Io> {
        BindingsContext {
            io: self.io,
            #[cfg(not(target_arch = "wasm32"))]
            wasi_table: ResourceTable::new(),
            #[cfg(not(target_arch = "wasm32"))]
            wasi_ctx: crate::wasi::WasiCtxBuilder::new().build(),
            #[cfg(not(target_arch = "wasm32"))]
            common_table: ResourceTable::new(),
        }
    }
}

pub struct BindingsImpl<T>(pub T);
impl<T: BindingsView> BindingsView for BindingsImpl<T> {
    type Io = T::Io;
    fn ctx(&self) -> &BindingsContext<Self::Io> {
        T::ctx(&self.0)
    }

    fn ctx_mut(&mut self) -> &mut BindingsContext<Self::Io> {
        T::ctx_mut(&mut self.0)
    }
}

impl<T: ?Sized + BindingsView> BindingsView for &mut T {
    type Io = T::Io;

    fn ctx(&self) -> &BindingsContext<Self::Io> {
        T::ctx(self)
    }

    fn ctx_mut(&mut self) -> &mut BindingsContext<Self::Io> {
        T::ctx_mut(self)
    }
}

/*
#[cfg(not(target_arch = "wasm32"))]
impl<T> crate::wasi::WasiView for T
where
    T: BindingsView,
{
    fn table(&mut self) -> &mut ResourceTable {
        &mut T::ctx_mut(self).wasi_table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut T::ctx_mut(self).wasi_ctx
    }
}
*/

/*
#[cfg(not(target_arch = "wasm32"))]
impl<T> crate::wasi::WasiView for &mut T
where
    T: BindingsView,
{
    fn table(&mut self) -> &mut ResourceTable {
        &mut T::ctx_mut(self).wasi_table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut T::ctx_mut(self).wasi_ctx
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> crate::wasi::WasiView for BindingsImpl<T>
where
    T: BindingsView,
{
    fn table(&mut self) -> &mut ResourceTable {
        &mut T::ctx_mut(&mut self.0).wasi_table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut T::ctx_mut(&mut self.0).wasi_ctx
    }
}
*/
