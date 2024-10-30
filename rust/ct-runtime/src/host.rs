use ct_common::ConditionalSend;

/// Callback executed when runtime invokes `callback`
/// in `common:basic/host-callback@0.0.1`.
pub trait HostCallbackFn:
    Fn(String) -> std::result::Result<String, String> + ConditionalSend + 'static
{
}
impl<T> HostCallbackFn for T where
    T: Fn(String) -> std::result::Result<String, String> + ConditionalSend + 'static
{
}

#[cfg(not(target_arch = "wasm32"))]
mod host_callback_impl {
    use super::HostCallbackFn;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct HostCallback(Arc<Mutex<Box<dyn HostCallbackFn>>>);

    impl HostCallback {
        pub fn new<T: HostCallbackFn>(value: T) -> Self {
            HostCallback(Arc::new(Mutex::new(Box::new(value))))
        }

        pub fn invoke(&self, input: String) -> std::result::Result<String, String> {
            let callback = self.0.lock().map_err(|e| e.to_string())?;
            callback(input)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod host_callback_impl {
    use super::HostCallbackFn;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct HostCallback(Arc<Box<dyn HostCallbackFn>>);

    impl HostCallback {
        pub fn new<T: HostCallbackFn>(value: T) -> Self {
            HostCallback(Arc::new(Box::new(value)))
        }

        pub fn invoke(&self, input: String) -> std::result::Result<String, String> {
            (self.0)(input)
        }
    }
}

pub(crate) use host_callback_impl::*;
