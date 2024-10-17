use crate::{module::ModuleContext, runtime::BasicIo, ModuleContextMut};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView};

/// The backing [ModuleContext] for a
/// [crate::target::function_vm::NativeFormulaVm] Module.
pub struct NativeFormulaVmContext {
    wasi_resources: ResourceTable,
    wasi_ctx: WasiCtx,
}

impl NativeFormulaVmContext {
    /// Create a new [`NativeFormulaVmContext`].
    pub fn new() -> Self {
        Self {
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

impl Default for NativeFormulaVmContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleContext for NativeFormulaVmContext {
    type Io = BasicIo;

    fn io(&self) -> &Self::Io {
        unimplemented!()
    }

    fn ifc(&self) -> &common_ifc::Context {
        unimplemented!()
    }
}

impl ModuleContextMut for NativeFormulaVmContext {
    fn io_mut(&mut self) -> &mut Self::Io {
        unimplemented!()
    }
}

impl WasiView for NativeFormulaVmContext {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}
