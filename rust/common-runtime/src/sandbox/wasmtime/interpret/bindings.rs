use crate::wasmtime::bindings::common_script::*;
use axum::async_trait;
use common_wasi::{WasiCtx, WasiView};
use wasmtime::component::{Resource, ResourceTable};

// NOTE: This module comes from wasmtime::component::bindgen
use common::data::types::{Reference, Value as BindingValue};

use crate::{InputOutput, Value};

impl From<BindingValue> for Value {
    fn from(value: BindingValue) -> Self {
        match value {
            BindingValue::String(inner) => Value::String(inner),
            BindingValue::Number(inner) => Value::Number(inner),
            BindingValue::Boolean(inner) => Value::Boolean(inner),
            BindingValue::Buffer(inner) => Value::Buffer(inner),
        }
    }
}

impl From<Value> for BindingValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(inner) => BindingValue::String(inner),
            Value::Boolean(inner) => BindingValue::Boolean(inner),
            Value::Number(inner) => BindingValue::Number(inner),
            Value::Buffer(inner) => BindingValue::Buffer(inner),
        }
    }
}

#[repr(transparent)]
struct ScriptHostReference(String);

/// A [ScriptHostState] embodies the bindings from a host runtime to that are
/// made available to a guest Common Script. The shape of these bindings are
/// determined by the Common WIT definitions. All traits that represent the
/// substantive implementation of a [ScriptHostState] are generated by
/// [wasmtime::component::bindgen].
pub struct ScriptHostState<Io: InputOutput> {
    io: Io,
    references: ResourceTable,

    view_resources: ResourceTable,
    view_ctx: WasiCtx,
}

impl<Io> ScriptHostState<Io>
where
    Io: InputOutput,
{
    fn guest_reference_to_host_reference(
        &self,
        reference: Resource<Reference>,
    ) -> Result<&ScriptHostReference, String> {
        let host_resource = Resource::<ScriptHostReference>::new_own(reference.rep());

        self.references
            .get(&host_resource)
            .map_err(|error| format!("{error}"))
    }

    /// Instantiate the [ScriptHostState] by providing it an implementor of
    /// [InputOutput] that will be made indirectly available to the guest Common
    /// Modules that are instantiated under the [ScriptHostState].
    pub fn new(io: Io) -> Self {
        ScriptHostState {
            io,
            references: ResourceTable::default(),

            view_resources: ResourceTable::default(),
            view_ctx: WasiCtx::builder().build(),
        }
    }

    pub fn io(&self) -> &Io {
        &self.io
    }

    pub fn take_io(&mut self) -> Io {
        std::mem::take(&mut self.io)
    }

    pub fn replace_io(&mut self, io: Io) {
        self.io = io;
    }
}

#[async_trait]
impl<Io> common::io::state::Host for ScriptHostState<Io>
where
    Io: InputOutput,
{
    async fn read(&mut self, name: String) -> Option<wasmtime::component::Resource<Reference>> {
        debug!("common:io/state.read: {name}");
        self.io.read(&name)?;

        self.references
            .push(ScriptHostReference(name))
            .map_err(|error| error!("Unable to allocate Reference: {error}"))
            .ok()
            .map(|host_reference| Resource::new_own(host_reference.rep()))
    }

    async fn write(&mut self, name: String, value: BindingValue) -> () {
        debug!("common:io/state.write: {name}");
        self.io.write(&name, value.into());
    }
}

#[async_trait]
impl<Io> common::data::types::HostReference for ScriptHostState<Io>
where
    Io: InputOutput,
{
    /// Dereference a reference to a value
    /// This call is fallible (for example, if the dereference is not allowed)
    /// The value may be none (for example, if it is strictly opaque)
    async fn deref(
        &mut self,
        resource: Resource<Reference>,
    ) -> Result<Option<BindingValue>, String> {
        let ScriptHostReference(key) = self.guest_reference_to_host_reference(resource)?;
        Ok(self.io.read(key).map(|value| value.into()))
    }

    fn drop(&mut self, rep: Resource<Reference>) -> wasmtime::Result<()> {
        let host_resource = Resource::<ScriptHostReference>::new_own(rep.rep());
        self.references.delete(host_resource)?;
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

impl<Io> common::data::types::Host for ScriptHostState<Io> where Io: InputOutput {}

#[async_trait]
impl<Io> common::module::reflect::Host for ScriptHostState<Io>
where
    Io: InputOutput,
{
    async fn input_keys(&mut self) -> Vec<String> {
        todo!("Input key enumeration not yet supported")
    }

    async fn output_keys(&mut self) -> Vec<String> {
        todo!("Output key enumeration not yet supported")
    }
}

impl<Io> WasiView for ScriptHostState<Io>
where
    Io: InputOutput,
{
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.view_resources
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.view_ctx
    }
}
