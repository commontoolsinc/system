use async_trait::async_trait;
use common_protos::runtime::{RunModuleRequest, RunModuleResponse};
use http::Uri;
use std::sync::Arc;

use crate::{
    remote::client::make_runtime_client, CommonRuntimeError, FunctionInterface, HasModuleContext,
    HasModuleContextMut, InputOutput, IoData, Module, ModuleContext, ModuleId, ModuleInstanceId,
    RemoteFunctionDefinition, Validated,
};

use super::WebRemoteFunctionContext;

/// An `common:function/module`-based Module facade for the [crate::WebRuntime].
pub struct WebRemoteFunction {
    module_id: ModuleId,
    instance_id: ModuleInstanceId,
    context: WebRemoteFunctionContext,
    remote_runtime_address: Uri,
}

impl WebRemoteFunction {
    /// Instantiate a [WebRemoteFunction] with a [crate::ModuleDefinition] and
    /// remote Runtime session properties
    pub fn new(
        definition: Arc<RemoteFunctionDefinition>,
        context: WebRemoteFunctionContext,
        instance_id: ModuleInstanceId,
        remote_runtime_address: Uri,
    ) -> Result<Self, CommonRuntimeError> {
        let module_id = ModuleId::from(&*(*definition));

        Ok(Self {
            module_id,
            instance_id,
            context,
            remote_runtime_address,
        })
    }
}

#[async_trait(?Send)]
impl FunctionInterface for WebRemoteFunction {
    type InputOutput = <WebRemoteFunctionContext as ModuleContext>::Io;

    async fn run(
        &mut self,
        io: Validated<Self::InputOutput>,
    ) -> Result<IoData, CommonRuntimeError> {
        let mut client = make_runtime_client(&self.remote_runtime_address);

        let RunModuleResponse { output } = client
            .run_module(RunModuleRequest {
                instance_id: self.instance_id.to_string(),
                input: io.into_inner().input().into(),
            })
            .await
            .map_err(|error| CommonRuntimeError::ModuleRunFailed(format!("{error}")))?
            .into_inner();

        Ok(IoData::try_from(output)?)
    }
}

impl Module for WebRemoteFunction {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn instance_id(&self) -> &ModuleInstanceId {
        &self.instance_id
    }
}

impl HasModuleContext for WebRemoteFunction {
    type Context = WebRemoteFunctionContext;

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

impl HasModuleContextMut for WebRemoteFunction {
    fn context_mut(&mut self) -> &mut Self::Context {
        &mut self.context
    }
}
