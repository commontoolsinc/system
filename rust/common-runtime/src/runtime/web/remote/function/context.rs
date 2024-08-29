use common_ifc::Context as IfcContext;

use crate::{BasicIo, ModuleContext, ModuleContextMut};

/// The backing [ModuleContext] for a
/// [crate::remote::function::WebRemoteFunction] Module
pub struct WebRemoteFunctionContext {
    io: BasicIo,
    ifc: IfcContext,
}

impl WebRemoteFunctionContext {
    /// Instantiate a new [WebRemoteFunctionContext] with a [BasicIo] and an
    /// [IfcContext]
    pub fn new(io: BasicIo, ifc: IfcContext) -> Self {
        Self { io, ifc }
    }
}

impl ModuleContext for WebRemoteFunctionContext {
    type Io = BasicIo;

    fn io(&self) -> &Self::Io {
        &self.io
    }

    fn ifc(&self) -> &IfcContext {
        &self.ifc
    }
}

impl ModuleContextMut for WebRemoteFunctionContext {
    fn io_mut(&mut self) -> &mut Self::Io {
        &mut self.io
    }
}
