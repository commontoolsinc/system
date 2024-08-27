use async_trait::async_trait;
use wasmtime::component::{bindgen, Resource, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiView};
use wasmtime_wasi_http::WasiHttpView;

use crate::{InputOutput, Value};

use super::NativeFunctionContext;

bindgen!({
  world: "module",
  path: "../../wit/common/function/wit",
  async: true
});

use common::data::types::{Reference, Value as GuestValue};

impl From<GuestValue> for Value {
    fn from(value: GuestValue) -> Self {
        match value {
            GuestValue::String(inner) => Value::String(inner),
            GuestValue::Number(inner) => Value::Number(inner),
            GuestValue::Boolean(inner) => Value::Boolean(inner),
            GuestValue::Buffer(inner) => Value::Buffer(inner),
        }
    }
}

impl From<Value> for GuestValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(inner) => GuestValue::String(inner),
            Value::Boolean(inner) => GuestValue::Boolean(inner),
            Value::Number(inner) => GuestValue::Number(inner),
            Value::Buffer(inner) => GuestValue::Buffer(inner),
        }
    }
}

#[repr(transparent)]
struct ModuleHostReference(String);

#[async_trait]
impl common::io::state::Host for NativeFunctionContext {
    async fn read(&mut self, name: String) -> Option<wasmtime::component::Resource<Reference>> {
        debug!("common:io/state.read: {name}");
        self.io.read(&name)?;

        self.resources
            .push(ModuleHostReference(name))
            .map_err(|error| error!("Unable to allocate Reference: {error}"))
            .ok()
            .map(|host_reference| Resource::new_own(host_reference.rep()))
    }

    async fn write(&mut self, name: String, value: GuestValue) -> () {
        debug!("common:io/state.write: {name}");
        self.io.write(&name, value.into());
    }
}

#[async_trait]
impl common::data::types::HostReference for NativeFunctionContext {
    /// Dereference a reference to a value
    /// This call is fallible (for example, if the dereference is not allowed)
    /// The value may be none (for example, if it is strictly opaque)
    async fn deref(&mut self, resource: Resource<Reference>) -> Result<Option<GuestValue>, String> {
        let host_resource = Resource::<ModuleHostReference>::new_own(resource.rep());

        let ModuleHostReference(key) = self
            .resources
            .get(&host_resource)
            .map_err(|error| format!("{error}"))?;

        Ok(self.io.read(key).map(|value| value.into()))
    }

    fn drop(&mut self, rep: Resource<Reference>) -> wasmtime::Result<()> {
        let host_resource = Resource::<ModuleHostReference>::new_own(rep.rep());
        self.resources.delete(host_resource)?;
        Ok(())
    }

    async fn read(
        &mut self,
        _this: wasmtime::component::Resource<Reference>,
        _name: String,
    ) -> Option<wasmtime::component::Resource<Reference>> {
        todo!("Resource sub-keys not yet supported")
    }
}

impl common::data::types::Host for NativeFunctionContext {}

#[async_trait]
impl common::function::reflect::Host for NativeFunctionContext {
    async fn input_keys(&mut self) -> Vec<String> {
        todo!("Input key enumeration not yet supported")
    }

    async fn output_keys(&mut self) -> Vec<String> {
        todo!("Output key enumeration not yet supported")
    }
}

impl WasiView for NativeFunctionContext {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl WasiHttpView for NativeFunctionContext {
    fn ctx(&mut self) -> &mut wasmtime_wasi_http::WasiHttpCtx {
        &mut self.wasi_http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.wasi_http_resources
    }
}
