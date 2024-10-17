//! Bindings for the Wasmtime runtime.
//!
//! Wasmtime bindings are implemented as traits against
//! some context that implements [WasiView] in order to
//! access underlying [WasiCtx] and [ResourceTable].
//! In order to be generic across context types, we roll
//! our own [BindingsView] to access Common Tools specific
//! resources.
//! A wrapper [BindingsImpl] provides a concrete implementation
//! to target these implementions, a pattern used by wasmtime's
//! [WASI bindings][wasi bindings], [WASI HTTP bindings][wasi-http bindings],
//! and [recommended to others][wasmtime-8764] pursuing similar.
//!
//! [wasi bindings]: https://github.com/bytecodealliance/wasmtime/blob/main/crates/wasi-common/src/lib.rs
//! [wasi-http bindings]: https://github.com/bytecodealliance/wasmtime/blob/main/crates/wasi-http/src/lib.rs
//! [wasmtime-8764]: https://github.com/bytecodealliance/wasmtime/issues/8764

use crate::{InputOutput, ModuleContext, ModuleContextMut, Value};
use async_trait::async_trait;
use wasmtime::component::Resource;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiView};

/// Helper function for linking contexts.
fn type_annotate<T: BindingsView, F>(val: F) -> F
where
    F: Fn(&mut T) -> BindingsImpl<&mut T>,
{
    val
}

/// Trait to implement for registering as
/// a wasmtime context.
pub trait BindingsView: WasiView + ModuleContext + ModuleContextMut {
    fn common_table(&self) -> &ResourceTable;
    fn common_table_mut(&mut self) -> &mut ResourceTable;
}

struct BindingsImpl<T>(pub T);

impl<T> WasiView for BindingsImpl<T>
where
    T: WasiView,
{
    fn ctx(&mut self) -> &mut WasiCtx {
        self.0.ctx()
    }
    fn table(&mut self) -> &mut ResourceTable {
        self.0.table()
    }
}

impl<T> BindingsView for BindingsImpl<T>
where
    T: BindingsView,
{
    fn common_table(&self) -> &ResourceTable {
        self.0.common_table()
    }
    fn common_table_mut(&mut self) -> &mut ResourceTable {
        self.0.common_table_mut()
    }
}

impl<T> BindingsView for &mut T
where
    T: BindingsView,
{
    fn common_table(&self) -> &ResourceTable {
        T::common_table(self)
    }
    fn common_table_mut(&mut self) -> &mut ResourceTable {
        T::common_table_mut(self)
    }
}

impl<T> ModuleContext for BindingsImpl<T>
where
    T: BindingsView,
{
    type Io = T::Io;

    fn ifc(&self) -> &common_ifc::Context {
        self.0.ifc()
    }
    fn io(&self) -> &Self::Io {
        self.0.io()
    }
}

impl<T> ModuleContextMut for BindingsImpl<T>
where
    T: BindingsView,
{
    fn io_mut(&mut self) -> &mut Self::Io {
        self.0.io_mut()
    }
}

#[repr(transparent)]
struct HostReference(String);

#[allow(missing_docs)]
pub mod module {
    wasmtime::component::bindgen!({
      world: "module",
      path: "../../wit/common/function/wit",
      async: true
    });

    /// Link resources for `common:function` targets.
    pub fn add_to_linker<T>(l: &mut wasmtime::component::Linker<T>) -> wasmtime::Result<()>
    where
        T: super::BindingsView,
    {
        let get = super::type_annotate::<T, _>(|t| super::BindingsImpl(t));
        // Manually link needed types -- we want to call
        // `add_to_linker_get_host()` on dependent types,
        // rather than `Module::add_to_linker()`,
        // due to bounds on these functions with `BindingsImpl`.
        common::data::types::add_to_linker_get_host(l, get)?;
        common::io::state::add_to_linker_get_host(l, get)?;
        common::function::reflect::add_to_linker_get_host(l, get)?;
        Ok(())
    }
}

#[allow(missing_docs)]
pub mod vm {
    wasmtime::component::bindgen!({
      world: "virtual-module",
      path: "../../wit/common/function/wit",
      async: true,
      with: {
          "common:function": super::module::common::function,
          "common:data": super::module::common::data,
          "common:io": super::module::common::io,
      }
    });

    /// Link resources for `common:function/virtual-module` targets.
    pub fn add_to_linker<T>(l: &mut wasmtime::component::Linker<T>) -> wasmtime::Result<()>
    where
        T: super::BindingsView,
    {
        let get = super::type_annotate::<T, _>(|t| super::BindingsImpl(t));
        // Manually link needed types -- we want to call
        // `add_to_linker_get_host()` on dependent types,
        // rather than `VirtualModule::add_to_linker()`,
        // due to bounds on these functions with `BindingsImpl`.
        common::data::types::add_to_linker_get_host(l, get)?;
        common::io::state::add_to_linker_get_host(l, get)?;
        common::function::reflect::add_to_linker_get_host(l, get)?;
        Ok(())
    }
}

use module::common::{
    self,
    data::types::{Reference, Value as GuestValue},
};

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

#[async_trait]
impl<T> common::io::state::Host for BindingsImpl<T>
where
    T: BindingsView,
{
    async fn read(&mut self, name: String) -> Option<wasmtime::component::Resource<Reference>> {
        debug!("common:io/state.read: {name}");
        self.io().read(&name)?;

        self.common_table_mut()
            .push(HostReference(name))
            .map_err(|error| error!("Unable to allocate Reference: {error}"))
            .ok()
            .map(|host_reference| Resource::new_own(host_reference.rep()))
    }

    async fn write(&mut self, name: String, value: GuestValue) -> () {
        debug!("common:io/state.write: {name}");
        self.io_mut().write(&name, value.into());
    }
}

#[async_trait]
impl<T> common::data::types::HostReference for BindingsImpl<T>
where
    T: BindingsView,
{
    /// Dereference a reference to a value
    /// This call is fallible (for example, if the dereference is not allowed)
    /// The value may be none (for example, if it is strictly opaque)
    async fn deref(&mut self, resource: Resource<Reference>) -> Result<Option<GuestValue>, String> {
        let host_resource = Resource::<HostReference>::new_own(resource.rep());

        let HostReference(key) = self
            .common_table()
            .get(&host_resource)
            .map_err(|error| format!("{error}"))?;

        Ok(self.io().read(key).map(|value| value.into()))
    }

    async fn drop(&mut self, rep: Resource<Reference>) -> wasmtime::Result<()> {
        let host_resource = Resource::<HostReference>::new_own(rep.rep());
        self.common_table_mut().delete(host_resource)?;
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

impl<T> common::data::types::Host for BindingsImpl<T> where T: BindingsView {}

#[async_trait]
impl<T> common::function::reflect::Host for BindingsImpl<T>
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
