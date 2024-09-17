use common_ifc::Policy;
use common_macros::NewType;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::ffi::web::cast::to_string;
use crate::ffi::web::host::JavaScriptValueMap;
use crate::{remote::function::WebRemoteFunction, Module, ModuleInstanceId};
use crate::{
    BasicIo, FunctionInterface, HasModuleContext, InputOutput, IoValues, ModuleContext, Validated,
};

/// All possible variants of things that implement [`FunctionInterface`]
#[derive(Clone)]
pub enum FunctionVariant {
    /// A remote function
    RemoteModule(Rc<RefCell<WebRemoteFunction>>),
}

impl FunctionVariant {
    /// Get the [`ModuleInstanceId`] for the interior function
    pub fn instance_id(&self) -> ModuleInstanceId {
        match self {
            FunctionVariant::RemoteModule(module) => module.borrow().instance_id().clone(),
        }
    }
}

/// A newtype over all possible variants of things that implement [`FunctionInterface`].
/// This is the concrete type that is exposed to the web browser host.
#[wasm_bindgen]
#[derive(NewType)]
pub struct CommonFunction(FunctionVariant);

#[wasm_bindgen]
impl CommonFunction {
    #[wasm_bindgen]
    /// Invoke the interior function
    pub async fn run(&self, input: JavaScriptValueMap) -> Result<JavaScriptValueMap, String> {
        match &self.0 {
            FunctionVariant::RemoteModule(remote_function) => {
                let mut function = remote_function.borrow_mut();
                let io = BasicIo::from_initial_state(
                    IoValues::from(input),
                    function.context().io().output_shape().clone(),
                );

                let validated_io = Validated::try_from((
                    Policy::with_defaults().map_err(to_string)?,
                    function.context().ifc(),
                    io,
                ))
                .map_err(to_string)?;

                let output = function.run(validated_io).await.map_err(to_string)?;
                let output_values = IoValues::from(&output);

                Ok(output_values.into())
            }
        }
    }
}
