use super::{FsError, FsResult};
use crate::wasi::bindings::filesystem::types::ErrorCode;
use crate::wasi::bindings::sync::filesystem::types as sync_filesystem;
use crate::wasi::bindings::sync::io::streams;
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

impl<T> sync_filesystem::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn convert_error_code(&mut self, _err: FsError) -> anyhow::Result<sync_filesystem::ErrorCode> {
        Err(ErrorCode::Unsupported.into())
    }

    fn filesystem_error_code(
        &mut self,
        _err: Resource<streams::Error>,
    ) -> anyhow::Result<Option<sync_filesystem::ErrorCode>> {
        Err(ErrorCode::Unsupported.into())
    }
}

impl<T> sync_filesystem::HostDescriptor for WasiImpl<T>
where
    T: WasiView,
{
    fn advise(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _offset: sync_filesystem::Filesize,
        _len: sync_filesystem::Filesize,
        _advice: sync_filesystem::Advice,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn sync_data(&mut self, _fd: Resource<sync_filesystem::Descriptor>) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn get_flags(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<sync_filesystem::DescriptorFlags> {
        Err(ErrorCode::Unsupported.into())
    }

    fn get_type(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<sync_filesystem::DescriptorType> {
        Err(ErrorCode::Unsupported.into())
    }

    fn set_size(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _size: sync_filesystem::Filesize,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn set_times(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _atim: sync_filesystem::NewTimestamp,
        _mtim: sync_filesystem::NewTimestamp,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn read(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _len: sync_filesystem::Filesize,
        _offset: sync_filesystem::Filesize,
    ) -> FsResult<(Vec<u8>, bool)> {
        Err(ErrorCode::Unsupported.into())
    }

    fn write(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _buf: Vec<u8>,
        _offset: sync_filesystem::Filesize,
    ) -> FsResult<sync_filesystem::Filesize> {
        Err(ErrorCode::Unsupported.into())
    }

    fn read_directory(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<Resource<sync_filesystem::DirectoryEntryStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn sync(&mut self, _fd: Resource<sync_filesystem::Descriptor>) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn create_directory_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn stat(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<sync_filesystem::DescriptorStat> {
        Err(ErrorCode::Unsupported.into())
    }

    fn stat_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path_flags: sync_filesystem::PathFlags,
        _path: String,
    ) -> FsResult<sync_filesystem::DescriptorStat> {
        Err(ErrorCode::Unsupported.into())
    }

    fn set_times_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path_flags: sync_filesystem::PathFlags,
        _path: String,
        _atim: sync_filesystem::NewTimestamp,
        _mtim: sync_filesystem::NewTimestamp,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn link_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        // TODO delete the path flags from this function
        _old_path_flags: sync_filesystem::PathFlags,
        _old_path: String,
        _new_descriptor: Resource<sync_filesystem::Descriptor>,
        _new_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn open_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path_flags: sync_filesystem::PathFlags,
        _path: String,
        _oflags: sync_filesystem::OpenFlags,
        _flags: sync_filesystem::DescriptorFlags,
    ) -> FsResult<Resource<sync_filesystem::Descriptor>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn drop(&mut self, _fd: Resource<sync_filesystem::Descriptor>) -> anyhow::Result<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn readlink_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path: String,
    ) -> FsResult<String> {
        Err(ErrorCode::Unsupported.into())
    }

    fn remove_directory_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn rename_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _old_path: String,
        _new_fd: Resource<sync_filesystem::Descriptor>,
        _new_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn symlink_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _src_path: String,
        _dest_path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn unlink_file_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path: String,
    ) -> FsResult<()> {
        Err(ErrorCode::Unsupported.into())
    }

    fn read_via_stream(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _offset: sync_filesystem::Filesize,
    ) -> FsResult<Resource<streams::InputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn write_via_stream(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _offset: sync_filesystem::Filesize,
    ) -> FsResult<Resource<streams::OutputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn append_via_stream(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<Resource<streams::OutputStream>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn is_same_object(
        &mut self,
        _a: Resource<sync_filesystem::Descriptor>,
        _b: Resource<sync_filesystem::Descriptor>,
    ) -> anyhow::Result<bool> {
        Err(ErrorCode::Unsupported.into())
    }
    fn metadata_hash(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
    ) -> FsResult<sync_filesystem::MetadataHashValue> {
        Err(ErrorCode::Unsupported.into())
    }
    fn metadata_hash_at(
        &mut self,
        _fd: Resource<sync_filesystem::Descriptor>,
        _path_flags: sync_filesystem::PathFlags,
        _path: String,
    ) -> FsResult<sync_filesystem::MetadataHashValue> {
        Err(ErrorCode::Unsupported.into())
    }
}

impl<T> sync_filesystem::HostDirectoryEntryStream for WasiImpl<T>
where
    T: WasiView,
{
    fn read_directory_entry(
        &mut self,
        _stream: Resource<sync_filesystem::DirectoryEntryStream>,
    ) -> FsResult<Option<sync_filesystem::DirectoryEntry>> {
        Err(ErrorCode::Unsupported.into())
    }

    fn drop(
        &mut self,
        _stream: Resource<sync_filesystem::DirectoryEntryStream>,
    ) -> anyhow::Result<()> {
        Err(ErrorCode::Unsupported.into())
    }
}
