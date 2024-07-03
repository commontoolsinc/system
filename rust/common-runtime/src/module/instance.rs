use async_trait::async_trait;
use std::str::FromStr;

use crate::{wasmtime::WasmtimePrebuiltModule, CommonRuntimeError, InputOutput, PreparedModule};
use blake3::Hash;

/// All implementations of [PreparedModule] collected into an enum
#[derive(Clone)]
pub enum ModuleInstance {
    /// A [WasmtimePrebuiltModule]
    WasmtimePrebuiltModule(WasmtimePrebuiltModule),
}

#[async_trait]
impl PreparedModule for ModuleInstance {
    async fn call(
        &self,
        io: Box<dyn InputOutput>,
    ) -> Result<Box<dyn InputOutput>, CommonRuntimeError> {
        match self {
            ModuleInstance::WasmtimePrebuiltModule(module) => module.call(io).await,
        }
    }
}

/// Given a module ID, produces a unique instance ID which may be used to
/// identify an instantiation of the associted module.
pub fn make_instance_id(module_id: &str) -> Result<String, CommonRuntimeError> {
    let millis = std::time::SystemTime::now()
        .elapsed()
        .map_err(|error| CommonRuntimeError::InternalError(format!("{error}")))?
        .as_millis();
    let hash = Hash::from_str(module_id)
        .map_err(|error| CommonRuntimeError::InternalError(format!("{error}")))?;
    let entropy = rand::random::<u64>();

    Ok(blake3::hash(
        &millis
            .to_le_bytes()
            .into_iter()
            .chain(hash.as_bytes().to_owned())
            .chain(entropy.to_le_bytes())
            .collect::<Vec<u8>>(),
    )
    .to_string())
}
