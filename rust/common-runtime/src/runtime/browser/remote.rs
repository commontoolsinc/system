use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use common_protos::{
    common,
    runtime::{self, runtime_client::RuntimeClient},
};
use common_wit::Target;
use http::Uri;
use tonic_web_wasm_client::Client;

use crate::{
    runtime::RuntimeIo, CommonRuntimeError, ModuleDefinition, ModuleInstanceId, ModuleSource,
    ToModuleSources, ToWasmComponent,
};

// TODO: Support `RemoteRuntime` use in non-`wasm32-unknown-unknown` cases

#[derive(Clone)]
pub struct RemoteRuntime {
    client: RefCell<RuntimeClient<Client>>,
    module_instances: BTreeMap<ModuleInstanceId, ()>,
}

impl RemoteRuntime {
    pub fn new(address: Uri) -> Self {
        let client = RefCell::new(RuntimeClient::new(Client::new(address.to_string())));
        // RemoteRuntime { client }

        todo!();
    }

    pub async fn interpret<
        Module: ModuleDefinition + ToModuleSources + ToWasmComponent + 'static,
    >(
        &mut self,
        module: Module,
        io: RuntimeIo,
    ) -> Result<ModuleInstanceId, CommonRuntimeError> {
        let mut client = self.client.get_mut();

        let module_source = ModuleSource {
            target: Target::CommonFunctionVm,
            source_code: module.to_module_sources().await?.ok_or_else(|| {
                CommonRuntimeError::InvalidInstantiationParameters("No source code provided".into())
            })?,
        };

        let response = client
            .instantiate_module(runtime::InstantiateModuleRequest {
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
                        module_source.into(),
                    ),
                ),
            })
            .await
            .map_err(|error| {
                CommonRuntimeError::ModuleInstantiationFailed(format!(
                    "Remote runtime status: {}",
                    error
                ))
            })?
            .into_inner();

        Ok(ModuleInstanceId(response.instance_id))
    }
}
