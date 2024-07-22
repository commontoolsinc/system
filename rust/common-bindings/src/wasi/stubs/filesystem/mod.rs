use crate::wasi::bindings::filesystem::types::{self, ErrorCode};
use crate::wasi::TrappableError;
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

mod async_io;
mod sync;

pub type FsResult<T> = Result<T, FsError>;
pub type FsError = TrappableError<types::ErrorCode>;

pub enum Descriptor {
    File,
    Dir,
}

pub struct ReaddirIterator(
    std::sync::Mutex<Box<dyn Iterator<Item = FsResult<types::DirectoryEntry>> + Send + 'static>>,
);

impl IntoIterator for ReaddirIterator {
    type Item = FsResult<types::DirectoryEntry>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + Send>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().unwrap()
    }
}

impl<T> crate::wasi::bindings::filesystem::preopens::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_directories(
        &mut self,
    ) -> Result<Vec<(Resource<types::Descriptor>, String)>, anyhow::Error> {
        Err(ErrorCode::Unsupported.into())
    }
}
