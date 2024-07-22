use crate::{bindings::common_module, ctx::BindingsImpl, BindingsView};
use async_trait::async_trait;
use common_module::common::data::types::{Reference, Value as BindingValue};
use common_wit::InputOutput;
use wasmtime::component::Resource;

#[repr(transparent)]
pub struct HostReference(String);

#[async_trait]
impl<T> common_module::common::io::state::Host for BindingsImpl<T>
where
    T: BindingsView,
{
    async fn read(&mut self, name: String) -> Option<wasmtime::component::Resource<Reference>> {
        debug!("common:io/state.read: {name}");
        self.ctx().io.read(&name)?;

        self.ctx_mut()
            .common_table
            .push(HostReference(name))
            .map_err(|error| error!("Unable to allocate Reference: {error}"))
            .ok()
            .map(|host_reference| Resource::new_own(host_reference.rep()))
    }

    async fn write(&mut self, name: String, value: BindingValue) -> () {
        debug!("common:io/state.write: {name}");
        self.ctx_mut().io.write(&name, value.into());
    }
}

#[async_trait]
impl<T> common_module::common::data::types::HostReference for BindingsImpl<T>
where
    T: BindingsView,
{
    /// Dereference a reference to a value
    /// This call is fallible (for example, if the dereference is not allowed)
    /// The value may be none (for example, if it is strictly opaque)
    async fn deref(
        &mut self,
        resource: Resource<Reference>,
    ) -> Result<Option<BindingValue>, String> {
        let HostReference(key) = self.guest_reference_to_host_reference(resource)?;
        Ok(self.ctx().io.read(key).map(|value| value.into()))
    }

    fn drop(&mut self, rep: Resource<Reference>) -> wasmtime::Result<()> {
        let host_resource = Resource::<HostReference>::new_own(rep.rep());
        self.ctx_mut().common_table.delete(host_resource)?;
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

impl<T> common_module::common::data::types::Host for BindingsImpl<T> where T: BindingsView {}

#[async_trait]
impl<T> common_module::common::module::reflect::Host for BindingsImpl<T>
where
    T: BindingsView,
{
    async fn input_keys(&mut self) -> Vec<String> {
        todo!("Input key enumeration not yet supported")
    }

    async fn output_keys(&mut self) -> Vec<String> {
        todo!("Output key enumeration not yet supported")
    }
}
