use crate::bindings::cli::{
    stderr, stdin, stdout, terminal_input, terminal_output, terminal_stderr, terminal_stdin,
    terminal_stdout,
};
use crate::bindings::io::streams;
use crate::pipe;
use crate::{HostInputStream, HostOutputStream, WasiImpl, WasiView};
use wasmtime::component::Resource;

/// A trait used to represent the standard input to a guest program.
///
/// This is used to implement various WASI APIs via the method implementations
/// below.
///
/// Built-in implementations are provided for
/// [`pipe::MemoryInputPipe`], and [`pipe::ClosedInputStream`].
pub trait StdinStream: Send {
    /// Creates a fresh stream which is reading stdin.
    ///
    /// Note that the returned stream must share state with all other streams
    /// previously created. Guests may create multiple handles to the same stdin
    /// and they should all be synchronized in their progress through the
    /// program's input.
    ///
    /// Note that this means that if one handle becomes ready for reading they
    /// all become ready for reading. Subsequently if one is read from it may
    /// mean that all the others are no longer ready for reading. This is
    /// basically a consequence of the way the WIT APIs are designed today.
    fn stream(&self) -> Box<dyn HostInputStream>;

    /// Returns whether this stream is backed by a TTY.
    fn isatty(&self) -> bool;
}

impl StdinStream for pipe::MemoryInputPipe {
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(self.clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}

impl StdinStream for pipe::ClosedInputStream {
    fn stream(&self) -> Box<dyn HostInputStream> {
        Box::new(*self)
    }

    fn isatty(&self) -> bool {
        false
    }
}

/// Similar to [`StdinStream`], except for output.
pub trait StdoutStream: Send {
    /// Returns a fresh new stream which can write to this output stream.
    ///
    /// Note that all output streams should output to the same logical source.
    /// This means that it's possible for each independent stream to acquire a
    /// separate "permit" to write and then act on that permit. Note that
    /// additionally at this time once a permit is "acquired" there's no way to
    /// release it, for example you can wait for readiness and then never
    /// actually write in WASI. This means that acquisition of a permit for one
    /// stream cannot discount the size of a permit another stream could
    /// obtain.
    ///
    /// Implementations must be able to handle this
    fn stream(&self) -> Box<dyn HostOutputStream>;

    /// Returns whether this stream is backed by a TTY.
    fn isatty(&self) -> bool;
}

impl StdoutStream for pipe::MemoryOutputPipe {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(self.clone())
    }

    fn isatty(&self) -> bool {
        false
    }
}

impl StdoutStream for pipe::SinkOutputStream {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(*self)
    }

    fn isatty(&self) -> bool {
        false
    }
}

impl StdoutStream for pipe::ClosedOutputStream {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(*self)
    }

    fn isatty(&self) -> bool {
        false
    }
}

impl<T> stdin::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_stdin(&mut self) -> Result<Resource<streams::InputStream>, anyhow::Error> {
        let stream = self.ctx().stdin.stream();
        Ok(self.table().push(streams::InputStream::Host(stream))?)
    }
}

impl<T> stdout::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_stdout(&mut self) -> Result<Resource<streams::OutputStream>, anyhow::Error> {
        let stream = self.ctx().stdout.stream();
        Ok(self.table().push(stream)?)
    }
}

impl<T> stderr::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_stderr(&mut self) -> Result<Resource<streams::OutputStream>, anyhow::Error> {
        let stream = self.ctx().stderr.stream();
        Ok(self.table().push(stream)?)
    }
}

pub struct TerminalInput;
pub struct TerminalOutput;

impl<T> terminal_input::Host for WasiImpl<T> where T: WasiView {}
impl<T> terminal_input::HostTerminalInput for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _r: Resource<TerminalInput>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Unsupported"))
    }
}
impl<T> terminal_output::Host for WasiImpl<T> where T: WasiView {}
impl<T> terminal_output::HostTerminalOutput for WasiImpl<T>
where
    T: WasiView,
{
    fn drop(&mut self, _r: Resource<TerminalOutput>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Unsupported"))
    }
}
impl<T> terminal_stdin::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_terminal_stdin(&mut self) -> anyhow::Result<Option<Resource<TerminalInput>>> {
        Err(anyhow::anyhow!("Unsupported"))
    }
}
impl<T> terminal_stdout::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_terminal_stdout(&mut self) -> anyhow::Result<Option<Resource<TerminalOutput>>> {
        Err(anyhow::anyhow!("Unsupported"))
    }
}
impl<T> terminal_stderr::Host for WasiImpl<T>
where
    T: WasiView,
{
    fn get_terminal_stderr(&mut self) -> anyhow::Result<Option<Resource<TerminalOutput>>> {
        Err(anyhow::anyhow!("Unsupported"))
    }
}
