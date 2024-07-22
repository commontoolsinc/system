use super::{FsError, FsResult};
use crate::wasi::bindings::filesystem::types::{
    self, ErrorCode, HostDescriptor, HostDirectoryEntryStream,
};
use crate::wasi::bindings::io::streams::{InputStream, OutputStream};
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

#[async_trait::async_trait]
impl<T> types::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn convert_error_code(&mut self, _err: FsError) -> anyhow::Result<ErrorCode> {
        Err(ErrorCode::Unsupported.into())
    }

    fn filesystem_error_code(
        &mut self,
        _err: Resource<anyhow::Error>,
    ) -> anyhow::Result<Option<ErrorCode>> {
        Err(ErrorCode::Unsupported.into())
    }
}

#[async_trait::async_trait]
impl<T> HostDescriptor for WasiImpl<T>
where
    T: WasiView,
{
    async fn advise(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _offset: types::Filesize,
        _len: types::Filesize,
        _advice: types::Advice,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn sync_data(&mut self, _fd: Resource<types::Descriptor>) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn get_flags(
        &mut self,
        _fd: Resource<types::Descriptor>,
    ) -> FsResult<types::DescriptorFlags> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn get_type(
        &mut self,
        _fd: Resource<types::Descriptor>,
    ) -> FsResult<types::DescriptorType> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn set_size(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _size: types::Filesize,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn set_times(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _atim: types::NewTimestamp,
        _mtim: types::NewTimestamp,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn read(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _len: types::Filesize,
        _offset: types::Filesize,
    ) -> FsResult<(Vec<u8>, bool)> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn write(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _buf: Vec<u8>,
        _offset: types::Filesize,
    ) -> FsResult<types::Filesize> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn read_directory(
        &mut self,
        _fd: Resource<types::Descriptor>,
    ) -> FsResult<Resource<types::DirectoryEntryStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn sync(&mut self, _fd: Resource<types::Descriptor>) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn create_directory_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn stat(&mut self, _fd: Resource<types::Descriptor>) -> FsResult<types::DescriptorStat> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn stat_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path_flags: types::PathFlags,
        _path: String,
    ) -> FsResult<types::DescriptorStat> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn set_times_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path_flags: types::PathFlags,
        _path: String,
        _atim: types::NewTimestamp,
        _mtim: types::NewTimestamp,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn link_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        // TODO delete the path flags from this function
        _old_path_flags: types::PathFlags,
        _old_path: String,
        _new_descriptor: Resource<types::Descriptor>,
        _new_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn open_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path_flags: types::PathFlags,
        _path: String,
        _oflags: types::OpenFlags,
        _flags: types::DescriptorFlags,
    ) -> FsResult<Resource<types::Descriptor>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn drop(&mut self, _fd: Resource<types::Descriptor>) -> anyhow::Result<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn readlink_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path: String,
    ) -> FsResult<String> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn remove_directory_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn rename_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _old_path: String,
        _new_fd: Resource<types::Descriptor>,
        _new_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn symlink_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _src_path: String,
        _dest_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn unlink_file_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn read_via_stream(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _offset: types::Filesize,
    ) -> FsResult<Resource<InputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn write_via_stream(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _offset: types::Filesize,
    ) -> FsResult<Resource<OutputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn append_via_stream(
        &mut self,
        _fd: Resource<types::Descriptor>,
    ) -> FsResult<Resource<OutputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    async fn is_same_object(
        &mut self,
        _a: Resource<types::Descriptor>,
        _b: Resource<types::Descriptor>,
    ) -> anyhow::Result<bool> {
        Err(ErrorCode::Unsupported.into())
    }
    async fn metadata_hash(
        &mut self,
        _fd: Resource<types::Descriptor>,
    ) -> FsResult<types::MetadataHashValue> {
        Err(ErrorCode::Unsupported.into())
    }
    async fn metadata_hash_at(
        &mut self,
        _fd: Resource<types::Descriptor>,
        _path_flags: types::PathFlags,
        _path: String,
    ) -> FsResult<types::MetadataHashValue> {
        Err(ErrorCode::Unsupported.into())
    }
}

#[async_trait::async_trait]
impl<T> HostDirectoryEntryStream for WasiImpl<T>
where
    T: WasiView,
{
    async fn read_directory_entry(
        &mut self,
        _stream: Resource<types::DirectoryEntryStream>,
    ) -> FsResult<Option<types::DirectoryEntry>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn drop(&mut self, _stream: Resource<types::DirectoryEntryStream>) -> anyhow::Result<()> {
        Err(ErrorCode::Unsupported.into())
    }
}
