use std::sync::Arc;

use super::{WebRemoteFunction, WebRemoteFunctionContext};
use crate::{
    remote::client::make_runtime_client, CommonRuntimeError, InputOutput, ModuleContext,
    ModuleFactory, ModuleInstanceId, RemoteFunctionDefinition,
};
use async_trait::async_trait;
use common_protos::{
    common,
    runtime::{InstantiateModuleRequest, InstantiateModuleResponse},
};
use http::Uri;

#[cfg(doc)]
use crate::{ModuleDefinition, WebRuntime};

/// An implementor of [`ModuleFactory`] for [`WebRemoteFunction`] Modules that may be
/// instantiated by a [`WebRuntime`]
#[derive(Clone)]
pub struct WebRemoteFunctionFactory {
    definition: Arc<RemoteFunctionDefinition>,
    remote_runtime_address: Uri,
}

impl WebRemoteFunctionFactory {
    /// Instantiate a new [`WebRemoteFunctionFactory`] for a given
    /// [`ModuleDefinition`] and various Wasm runtime acoutrement
    pub fn new(definition: RemoteFunctionDefinition, remote_runtime_address: Uri) -> Self {
        Self {
            remote_runtime_address,
            definition: Arc::new(definition),
        }
    }
}

#[async_trait(?Send)]
impl ModuleFactory for WebRemoteFunctionFactory {
    type Context = WebRemoteFunctionContext;

    type Module = WebRemoteFunction;

    async fn instantiate(
        &self,
        context: Self::Context,
    ) -> Result<Self::Module, crate::CommonRuntimeError> {
        let mut client = make_runtime_client(&self.remote_runtime_address);

        let InstantiateModuleResponse { instance_id } = client
            .instantiate_module(InstantiateModuleRequest {
                output_shape: context.io().output_shape().into(),
                default_input: context.io().input().into(),
                target: common::Target::from(&self.definition.inner().target).into(),
                module_reference: Some(self.definition.inner().body.to_owned().into()),
            })
            .await
            .map_err(|error| CommonRuntimeError::ModuleInstantiationFailed(format!("{error}")))?
            .into_inner();

        let instance_id = ModuleInstanceId::from(instance_id);

        Ok(WebRemoteFunction::new(
            self.definition.clone(),
            context,
            instance_id,
            self.remote_runtime_address.clone(),
        )?)
    }
}
