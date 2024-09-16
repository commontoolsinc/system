use crate::{
    module::ModuleContext, runtime::BasicIo, target::bindings::BindingsView, ModuleContextMut,
};
use common_ifc::Context as IfcContext;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

/// The backing [ModuleContext] for a [crate::target::function::NativeFunction]
/// Module
pub struct NativeFunctionContext {
    io: BasicIo,

    ifc: IfcContext,

    resources: ResourceTable,

    wasi_resources: ResourceTable,
    wasi_ctx: WasiCtx,

    wasi_http_resources: ResourceTable,
    wasi_http_ctx: WasiHttpCtx,
}

impl NativeFunctionContext {
    /// Instantiate a new [NativeFunctionContext] with a [BasicIo] and an
    /// [IfcContext]
    pub fn new(io: BasicIo, ifc: IfcContext) -> Self {
        Self {
            io,
            ifc,
            resources: ResourceTable::new(),

            wasi_http_resources: ResourceTable::new(),
            wasi_http_ctx: WasiHttpCtx::new(),

            wasi_resources: ResourceTable::new(),
            wasi_ctx: WasiCtx::builder()
                .allow_tcp(false)
                .allow_udp(false)
                .allow_ip_name_lookup(false)
                .allow_blocking_current_thread(false)
                .inherit_stdout()
                .build(),
        }
    }
}

impl ModuleContext for NativeFunctionContext {
    type Io = BasicIo;

    fn io(&self) -> &Self::Io {
        &self.io
    }

    fn ifc(&self) -> &common_ifc::Context {
        &self.ifc
    }
}

impl ModuleContextMut for NativeFunctionContext {
    fn io_mut(&mut self) -> &mut Self::Io {
        &mut self.io
    }
}

impl BindingsView for NativeFunctionContext {
    fn common_table(&self) -> &ResourceTable {
        &self.resources
    }

    fn common_table_mut(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl WasiView for NativeFunctionContext {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl WasiHttpView for NativeFunctionContext {
    fn ctx(&mut self) -> &mut wasmtime_wasi_http::WasiHttpCtx {
        &mut self.wasi_http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_http_resources
    }
}
