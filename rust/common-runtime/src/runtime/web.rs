use async_trait::async_trait;
use http::Uri;
use remote::function::WebRemoteFunctionFactory;

use crate::{Cache, CommonRuntimeError, ModuleDriver, ModuleId, RemoteFunctionDefinition};

pub mod remote;

/// A Runtime implementation for "web" contexts, where "web" generally refers to
/// unmodified web browsers.
///
/// A [WebRuntime] implements Module types that are either facades over
/// counterpart implementations in remote Runtimes, or else
/// compatibility-patched modules that run in a local Wasm runtime. Wasm
/// runtimes on web browsers at the time of this writing do not natively support
/// Wasm Components, so the [WebRuntime] internally transpiles Modules so that
/// they work on the web.
pub struct WebRuntime {
    remote_runtime_address: Option<Uri>,
    remote_function_cache: Cache<ModuleId, WebRemoteFunctionFactory>,
}

impl WebRuntime {
    /// Instantiate the [WebRuntime], passing an optional address to an
    /// available remote Runtime that should be used when preparing,
    /// instantiating and invoking remote Modules
    pub fn new(remote_runtime_address: Option<Uri>) -> Result<WebRuntime, CommonRuntimeError> {
        Ok(Self {
            remote_runtime_address,
            remote_function_cache: Cache::new(32)?,
        })
    }
}

#[async_trait(?Send)]
impl ModuleDriver<RemoteFunctionDefinition> for WebRuntime {
    type ModuleFactory = WebRemoteFunctionFactory;

    async fn prepare(
        &self,
        definition: RemoteFunctionDefinition,
    ) -> Result<Self::ModuleFactory, CommonRuntimeError> {
        let module_id = ModuleId::from(definition.inner());

        let factory = if let Some(cached_item) = self.remote_function_cache.get(&module_id).await {
            cached_item
        } else {
            let factory = match &self.remote_runtime_address {
                Some(remote_runtime_address) => Ok(WebRemoteFunctionFactory::new(
                    definition,
                    remote_runtime_address.clone(),
                )),
                None => Err(CommonRuntimeError::PreparationFailed(
                    "Cannot prepare remote function; no remote runtime address was configured"
                        .to_string(),
                )),
            }?;

            self.remote_function_cache
                .insert(module_id, factory.clone())
                .await;

            factory
        };

        Ok(factory)
    }
}
