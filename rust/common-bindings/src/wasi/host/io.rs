use crate::wasi::{
    bindings::io::error,
    bindings::io::streams::{self, InputStream, OutputStream},
    poll::subscribe,
    Pollable, StreamError, StreamResult,
};
use crate::wasi::{WasiImpl, WasiView};
use wasmtime::component::Resource;

impl<T> error::Host for WasiImpl<T> where T: WasiView {}

impl<T> streams::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn convert_stream_error(&mut self, err: StreamError) -> anyhow::Result<streams::StreamError> {
        match err {
            StreamError::Closed => Ok(streams::StreamError::Closed),
            StreamError::LastOperationFailed(e) => Ok(streams::StreamError::LastOperationFailed(
                self.table().push(e)?,
            )),
            StreamError::Trap(e) => Err(e),
        }
    }
}

impl<T> error::HostError for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, err: Resource<streams::Error>) -> anyhow::Result<()> {
        self.table().delete(err)?;
        Ok(())
    }

    fn to_debug_string(&mut self, err: Resource<streams::Error>) -> anyhow::Result<String> {
        Ok(format!("{:?}", self.table().get(&err)?))
    }
}

#[async_trait::async_trait]
impl<T> streams::HostOutputStream for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, stream: Resource<OutputStream>) -> anyhow::Result<()> {
        self.table().delete(stream)?;
        Ok(())
    }

    fn check_write(&mut self, stream: Resource<OutputStream>) -> StreamResult<u64> {
        let bytes = self.table().get_mut(&stream)?.check_write()?;
        Ok(bytes as u64)
    }

    fn write(&mut self, stream: Resource<OutputStream>, bytes: Vec<u8>) -> StreamResult<()> {
        self.table().get_mut(&stream)?.write(bytes.into())?;
        Ok(())
    }

    fn subscribe(&mut self, stream: Resource<OutputStream>) -> anyhow::Result<Resource<Pollable>> {
        subscribe(self.table(), stream)
    }

    async fn blocking_write_and_flush(
        &mut self,
        stream: Resource<OutputStream>,
        bytes: Vec<u8>,
    ) -> StreamResult<()> {
        let s = self.table().get_mut(&stream)?;

        if bytes.len() > 4096 {
            return Err(StreamError::trap(
                "Buffer too large for blocking-write-and-flush (expected at most 4096)",
            ));
        }

        let mut bytes = bytes::Bytes::from(bytes);
        loop {
            let permit = s.write_ready().await?;
            let len = bytes.len().min(permit);
            let chunk = bytes.split_to(len);
            s.write(chunk)?;
            if bytes.is_empty() {
                break;
            }
        }

        s.flush()?;
        s.write_ready().await?;

        Ok(())
    }

    async fn blocking_write_zeroes_and_flush(
        &mut self,
        stream: Resource<OutputStream>,
        len: u64,
    ) -> StreamResult<()> {
        let s = self.table().get_mut(&stream)?;

        if len > 4096 {
            return Err(StreamError::trap(
                "Buffer too large for blocking-write-zeroes-and-flush (expected at most 4096)",
            ));
        }

        let mut len = len;
        while len > 0 {
            let permit = s.write_ready().await?;
            let this_len = len.min(permit as u64);
            s.write_zeroes(this_len as usize)?;
            len -= this_len;
        }

        s.flush()?;
        s.write_ready().await?;

        Ok(())
    }

    fn write_zeroes(&mut self, stream: Resource<OutputStream>, len: u64) -> StreamResult<()> {
        self.table().get_mut(&stream)?.write_zeroes(len as usize)?;
        Ok(())
    }

    fn flush(&mut self, stream: Resource<OutputStream>) -> StreamResult<()> {
        self.table().get_mut(&stream)?.flush()?;
        Ok(())
    }

    async fn blocking_flush(&mut self, stream: Resource<OutputStream>) -> StreamResult<()> {
        let s = self.table().get_mut(&stream)?;
        s.flush()?;
        s.write_ready().await?;
        Ok(())
    }

    async fn splice(
        &mut self,
        dest: Resource<OutputStream>,
        src: Resource<InputStream>,
        len: u64,
    ) -> StreamResult<u64> {
        let len = len.try_into().unwrap_or(usize::MAX);

        let permit = {
            let output = self.table().get_mut(&dest)?;
            output.check_write()?
        };
        let len = len.min(permit);
        if len == 0 {
            return Ok(0);
        }

        let contents = match self.table().get_mut(&src)? {
            InputStream::Host(h) => h.read(len)?,
            InputStream::File => return Err(StreamError::trap("Unsupported")),
        };

        let len = contents.len();
        if len == 0 {
            return Ok(0);
        }

        let output = self.table().get_mut(&dest)?;
        output.write(contents)?;
        Ok(len.try_into().expect("usize can fit in u64"))
    }

    async fn blocking_splice(
        &mut self,
        dest: Resource<OutputStream>,
        src: Resource<InputStream>,
        len: u64,
    ) -> StreamResult<u64> {
        use crate::wasi::Subscribe;

        self.table().get_mut(&dest)?.ready().await;

        self.table().get_mut(&src)?.ready().await;

        self.splice(dest, src, len).await
    }
}

#[async_trait::async_trait]
impl<T> streams::HostInputStream for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, stream: Resource<InputStream>) -> anyhow::Result<()> {
        self.table().delete(stream)?;
        Ok(())
    }

    async fn read(&mut self, stream: Resource<InputStream>, len: u64) -> StreamResult<Vec<u8>> {
        let len = len.try_into().unwrap_or(usize::MAX);
        let bytes = match self.table().get_mut(&stream)? {
            InputStream::Host(s) => s.read(len)?,
            InputStream::File => return Err(StreamError::trap("Unsupported")),
        };
        debug_assert!(bytes.len() <= len);
        Ok(bytes.into())
    }

    async fn blocking_read(
        &mut self,
        stream: Resource<InputStream>,
        len: u64,
    ) -> StreamResult<Vec<u8>> {
        if let InputStream::Host(s) = self.table().get_mut(&stream)? {
            s.ready().await;
        }
        self.read(stream, len).await
    }

    async fn skip(&mut self, stream: Resource<InputStream>, len: u64) -> StreamResult<u64> {
        let len = len.try_into().unwrap_or(usize::MAX);
        let written = match self.table().get_mut(&stream)? {
            InputStream::Host(s) => s.skip(len)?,
            InputStream::File => return Err(StreamError::trap("Unsupported")),
        };
        Ok(written.try_into().expect("usize always fits in u64"))
    }

    async fn blocking_skip(
        &mut self,
        stream: Resource<InputStream>,
        len: u64,
    ) -> StreamResult<u64> {
        if let InputStream::Host(s) = self.table().get_mut(&stream)? {
            s.ready().await;
        }
        self.skip(stream, len).await
    }

    fn subscribe(&mut self, stream: Resource<InputStream>) -> anyhow::Result<Resource<Pollable>> {
        crate::wasi::poll::subscribe(self.table(), stream)
    }
}

pub mod sync {
    use crate::wasi::{
        bindings::io::streams::{
            self as async_streams, Host as AsyncHost, HostInputStream as AsyncHostInputStream,
            HostOutputStream as AsyncHostOutputStream,
        },
        bindings::sync::io::poll::Pollable,
        bindings::sync::io::streams::{self, InputStream, OutputStream},
        runtime::in_tokio,
        StreamError, StreamResult,
    };
    use crate::wasi::{WasiImpl, WasiView};
    use wasmtime::component::Resource;

    impl From<async_streams::StreamError> for streams::StreamError {
        fn from(other: async_streams::StreamError) -> Self {
            match other {
                async_streams::StreamError::LastOperationFailed(e) => Self::LastOperationFailed(e),
                async_streams::StreamError::Closed => Self::Closed,
            }
        }
    }

    impl<T> streams::Host for WasiImpl<T>
    where
        T: WasiView,
    {
        fn convert_stream_error(
            &mut self,
            err: StreamError,
        ) -> anyhow::Result<streams::StreamError> {
            Ok(AsyncHost::convert_stream_error(self, err)?.into())
        }
    }

    impl<T> streams::HostOutputStream for WasiImpl<T>
    where
        T: WasiView,
    {
        fn drop(&mut self, stream: Resource<OutputStream>) -> anyhow::Result<()> {
            AsyncHostOutputStream::drop(self, stream)
        }

        fn check_write(&mut self, stream: Resource<OutputStream>) -> StreamResult<u64> {
            AsyncHostOutputStream::check_write(self, stream)
        }

        fn write(&mut self, stream: Resource<OutputStream>, bytes: Vec<u8>) -> StreamResult<()> {
            AsyncHostOutputStream::write(self, stream, bytes)
        }

        fn blocking_write_and_flush(
            &mut self,
            stream: Resource<OutputStream>,
            bytes: Vec<u8>,
        ) -> StreamResult<()> {
            in_tokio(async {
                AsyncHostOutputStream::blocking_write_and_flush(self, stream, bytes).await
            })
        }

        fn blocking_write_zeroes_and_flush(
            &mut self,
            stream: Resource<OutputStream>,
            len: u64,
        ) -> StreamResult<()> {
            in_tokio(async {
                AsyncHostOutputStream::blocking_write_zeroes_and_flush(self, stream, len).await
            })
        }

        fn subscribe(
            &mut self,
            stream: Resource<OutputStream>,
        ) -> anyhow::Result<Resource<Pollable>> {
            AsyncHostOutputStream::subscribe(self, stream)
        }

        fn write_zeroes(&mut self, stream: Resource<OutputStream>, len: u64) -> StreamResult<()> {
            AsyncHostOutputStream::write_zeroes(self, stream, len)
        }

        fn flush(&mut self, stream: Resource<OutputStream>) -> StreamResult<()> {
            AsyncHostOutputStream::flush(self, Resource::new_borrow(stream.rep()))
        }

        fn blocking_flush(&mut self, stream: Resource<OutputStream>) -> StreamResult<()> {
            in_tokio(async {
                AsyncHostOutputStream::blocking_flush(self, Resource::new_borrow(stream.rep()))
                    .await
            })
        }

        fn splice(
            &mut self,
            dst: Resource<OutputStream>,
            src: Resource<InputStream>,
            len: u64,
        ) -> StreamResult<u64> {
            in_tokio(async { AsyncHostOutputStream::splice(self, dst, src, len).await })
        }

        fn blocking_splice(
            &mut self,
            dst: Resource<OutputStream>,
            src: Resource<InputStream>,
            len: u64,
        ) -> StreamResult<u64> {
            in_tokio(async { AsyncHostOutputStream::blocking_splice(self, dst, src, len).await })
        }
    }

    impl<T> streams::HostInputStream for WasiImpl<T>
    where
        T: WasiView,
    {
        fn drop(&mut self, stream: Resource<InputStream>) -> anyhow::Result<()> {
            AsyncHostInputStream::drop(self, stream)
        }

        fn read(&mut self, stream: Resource<InputStream>, len: u64) -> StreamResult<Vec<u8>> {
            in_tokio(async { AsyncHostInputStream::read(self, stream, len).await })
        }

        fn blocking_read(
            &mut self,
            stream: Resource<InputStream>,
            len: u64,
        ) -> StreamResult<Vec<u8>> {
            in_tokio(async { AsyncHostInputStream::blocking_read(self, stream, len).await })
        }

        fn skip(&mut self, stream: Resource<InputStream>, len: u64) -> StreamResult<u64> {
            in_tokio(async { AsyncHostInputStream::skip(self, stream, len).await })
        }

        fn blocking_skip(&mut self, stream: Resource<InputStream>, len: u64) -> StreamResult<u64> {
            in_tokio(async { AsyncHostInputStream::blocking_skip(self, stream, len).await })
        }

        fn subscribe(
            &mut self,
            stream: Resource<InputStream>,
        ) -> anyhow::Result<Resource<Pollable>> {
            AsyncHostInputStream::subscribe(self, stream)
        }
    }
}
