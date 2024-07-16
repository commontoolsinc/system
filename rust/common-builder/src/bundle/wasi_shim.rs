use crate::BuilderError;
use anyhow::Result;

/// Mappings from [crate::bundle::polyfill] generated WASI
/// imports, to an identifier that can be replaced with
/// a working shim.
pub static WASI_MAPPINGS: [(&str, &str); 7] = [
    ("wasi:cli/*", "common:wasi-shim/cli.js#*"),
    ("wasi:clocks/*", "common:wasi-shim/clocks.js#*"),
    ("wasi:filesystem/*", "common:wasi-shim/filesystem.js#*"),
    ("wasi:http/*", "common:wasi-shim/http.js#*"),
    ("wasi:io/*", "common:wasi-shim/io.js#*"),
    ("wasi:random/*", "common:wasi-shim/random.js#*"),
    ("wasi:sockets/*", "common:wasi-shim/sockets.js#*"),
];

/// Represents a WASI 0.2 interface implemented in JS
/// that can be turned into a `&'static[u8]` for
/// including in polyfilled builds.
#[derive(Debug)]
pub enum JavaScriptWasiShim {
    Clocks,
    Filesystem,
    Http,
    Io,
    Random,
    Sockets,
    Cli,
}

impl JavaScriptWasiShim {
    /// Maps an ES import specifier to a [JavaScriptWasiShim].
    pub fn from_import_specifier(specifier: &str) -> Result<Self, BuilderError> {
        match specifier {
            "common:wasi-shim/clocks.js" => Ok(Self::Clocks),
            "common:wasi-shim/filesystem.js" => Ok(Self::Filesystem),
            "common:wasi-shim/http.js" => Ok(Self::Http),
            "common:wasi-shim/io.js" => Ok(Self::Io),
            "common:wasi-shim/random.js" => Ok(Self::Random),
            "common:wasi-shim/sockets.js" => Ok(Self::Sockets),
            "common:wasi-shim/cli.js" => Ok(Self::Cli),
            _ => Err(BuilderError::ModuleNotFound),
        }
    }
}

impl TryFrom<JavaScriptWasiShim> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(value: JavaScriptWasiShim) -> Result<Self> {
        static WASI_CLOCKS: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_CLOCKS"));
        static WASI_FILESYSTEM: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_FILESYSTEM"));
        static WASI_HTTP: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_HTTP"));
        static WASI_IO: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_IO"));
        static WASI_RANDOM: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_RANDOM"));
        static WASI_SOCKETS: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_SOCKETS"));
        static WASI_CLI: &[u8] = include_bytes!(env!("COMMON_WASI_SHIM_CLI"));

        let source_bytes = match value {
            JavaScriptWasiShim::Clocks => WASI_CLOCKS,
            JavaScriptWasiShim::Filesystem => WASI_FILESYSTEM,
            JavaScriptWasiShim::Http => WASI_HTTP,
            JavaScriptWasiShim::Io => WASI_IO,
            JavaScriptWasiShim::Random => WASI_RANDOM,
            JavaScriptWasiShim::Sockets => WASI_SOCKETS,
            JavaScriptWasiShim::Cli => WASI_CLI,
        }
        .to_vec();

        // The loader doesn't resolve relative paths from its specifier.
        // TODO: Look into this more to see if it can, or handle when
        // rolling our own stubs that aren't simply copies of
        // @bytecodealliance/preview2-shim
        Ok(String::from_utf8(source_bytes)?
            .replace("'./clocks.js'", "'common:wasi-shim/clocks.js'")
            .replace("'./filesystem.js'", "'common:wasi-shim/filesystem.js'")
            .replace("'./http.js'", "'common:wasi-shim/http.js'")
            .replace("'./io.js'", "'common:wasi-shim/io.js'")
            .replace("'./random.js'", "'common:wasi-shim/random.js'")
            .replace("'./sockets.js'", "'common:wasi-shim/sockets.js'")
            .replace("'./cli.js'", "'common:wasi-shim/cli.js'")
            .into_bytes())
    }
}
