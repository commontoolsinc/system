use crate::{
    module::ModuleContext, runtime::BasicIo, target::function_bindings::BindingsView,
    ModuleContextMut,
};
use common_ifc::Context as IfcContext;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView};

/// The backing [ModuleContext] for a
/// [crate::target::function_vm::NativeFunctionVm] Module.
pub struct NativeFunctionVmContext {
    io: BasicIo,
    ifc: IfcContext,

    resources: ResourceTable,

    wasi_resources: ResourceTable,
    wasi_ctx: WasiCtx,
}

impl NativeFunctionVmContext {
    /// Instantiate a new [NativeFunctionVmContext] with a [BasicIo] and an
    /// [IfcContext]
    pub fn new(io: BasicIo, ifc: IfcContext) -> Self {
        Self {
            io,
            ifc,
            resources: ResourceTable::new(),

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

impl ModuleContext for NativeFunctionVmContext {
    type Io = BasicIo;

    fn io(&self) -> &Self::Io {
        &self.io
    }

    fn ifc(&self) -> &common_ifc::Context {
        &self.ifc
    }
}

impl ModuleContextMut for NativeFunctionVmContext {
    fn io_mut(&mut self) -> &mut Self::Io {
        &mut self.io
    }
}

impl BindingsView for NativeFunctionVmContext {
    fn common_table(&self) -> &ResourceTable {
        &self.resources
    }

    fn common_table_mut(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl WasiView for NativeFunctionVmContext {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
