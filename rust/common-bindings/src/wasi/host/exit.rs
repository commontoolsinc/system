use crate::wasi::{bindings::cli::exit, I32Exit};
use crate::wasi::{WasiImpl, WasiView};

impl<T> exit::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn exit(&mut self, status: Result<(), ()>) -> anyhow::Result<()> {
        let status = match status {
            Ok(()) => 0,
            Err(()) => 1,
        };
        Err(anyhow::anyhow!(I32Exit(status)))
    }
}
