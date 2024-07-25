use std::{cell::RefCell, rc::Rc};

use common_protos::{
    common,
    runtime::{self, runtime_client::RuntimeClient},
};
use http::Uri;
use tonic_web_wasm_client::Client;

use crate::{
    runtime::RuntimeIo, CommonRuntimeError, ModuleDefinition, ModuleInstanceId, ToModuleSources,
    ToWasmComponent,
};

// TODO: Support `RemoteRuntime` use in non-`wasm32-unknown-unknown` cases

#[derive(Clone)]
pub struct RemoteRuntime {
    client: RefCell<RuntimeClient<Client>>,
}

impl RemoteRuntime {
    pub fn new(address: Uri) -> Self {
        let client = RefCell::new(RuntimeClient::new(Client::new(address.to_string())));
        RemoteRuntime { client }
    }

    pub async fn interpret<
        Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    >(
        &mut self,
        module: Module,
        io: RuntimeIo,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let mut client = self.client.get_mut();

        client.instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [("bar".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "foo".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("initial foo".into())),
                },
            )]
            .into(),
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSource(
                    common::ModuleSource {
                        target: common::Target::CommonModule.into(),
                        source_code: todo!(),
                    },
                ),
            ),
        });

        todo!()
    }
}
