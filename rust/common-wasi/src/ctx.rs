use crate::{
    clocks::{
        host::{monotonic_clock, wall_clock},
        HostMonotonicClock, HostWallClock,
    },
    pipe, random,
    stdio::{StdinStream, StdoutStream},
};
use cap_rand::{Rng, RngCore, SeedableRng};
use wasmtime::component::ResourceTable;

/// Builder-style structure used to create a [`WasiCtx`].
///
/// This type is used to create a [`WasiCtx`] that is considered per-[`Store`]
/// state. The [`build`][WasiCtxBuilder::build] method is used to finish the
/// building process and produce a finalized [`WasiCtx`].
///
/// # Examples
///
/// ```
/// use common_wasi::{WasiCtxBuilder, WasiCtx};
///
/// let mut wasi = WasiCtxBuilder::new();
/// wasi.arg("./foo.wasm");
/// wasi.arg("--help");
/// wasi.env("FOO", "bar");
///
/// let wasi: WasiCtx = wasi.build();
/// ```
///
/// [`Store`]: wasmtime::Store
pub struct WasiCtxBuilder {
    stdin: Box<dyn StdinStream>,
    stdout: Box<dyn StdoutStream>,
    stderr: Box<dyn StdoutStream>,
    env: Vec<(String, String)>,
    args: Vec<String>,
    random: Box<dyn RngCore + Send>,
    insecure_random: Box<dyn RngCore + Send>,
    insecure_random_seed: u128,
    wall_clock: Box<dyn HostWallClock + Send>,
    monotonic_clock: Box<dyn HostMonotonicClock + Send>,
    built: bool,
}

impl Default for WasiCtxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiCtxBuilder {
    /// Creates a builder for a new context with default parameters set.
    ///
    /// The current defaults are:
    ///
    /// * stdin is closed
    /// * stdout and stderr eat all input and it doesn't go anywhere
    /// * no env vars
    /// * no arguments
    /// * clocks use the host implementation of wall/monotonic clocks
    /// * RNGs are all initialized with random state and suitable generator
    ///   quality to satisfy the requirements of WASI APIs.
    ///
    /// These defaults can all be updated via the various builder configuration
    /// methods below.
    pub fn new() -> Self {
        // For the insecure random API, use `SmallRng`, which is fast. It's
        // also insecure, but that's the deal here.
        let insecure_random = Box::new(
            cap_rand::rngs::SmallRng::from_rng(cap_rand::thread_rng(cap_rand::ambient_authority()))
                .unwrap(),
        );

        // For the insecure random seed, use a `u128` generated from
        // `thread_rng()`, so that it's not guessable from the insecure_random
        // API.
        let insecure_random_seed =
            cap_rand::thread_rng(cap_rand::ambient_authority()).gen::<u128>();
        Self {
            stdin: Box::new(pipe::ClosedInputStream),
            stdout: Box::new(pipe::SinkOutputStream),
            stderr: Box::new(pipe::SinkOutputStream),
            env: Vec::new(),
            args: Vec::new(),
            random: random::thread_rng(),
            insecure_random,
            insecure_random_seed,
            wall_clock: wall_clock(),
            monotonic_clock: monotonic_clock(),
            built: false,
        }
    }

    /// Provides a custom implementation of stdin to use.
    pub fn stdin(&mut self, stdin: impl StdinStream + 'static) -> &mut Self {
        self.stdin = Box::new(stdin);
        self
    }

    /// Same as [`stdin`](WasiCtxBuilder::stdin), but for stdout.
    pub fn stdout(&mut self, stdout: impl StdoutStream + 'static) -> &mut Self {
        self.stdout = Box::new(stdout);
        self
    }

    /// Same as [`stdin`](WasiCtxBuilder::stdin), but for stderr.
    pub fn stderr(&mut self, stderr: impl StdoutStream + 'static) -> &mut Self {
        self.stderr = Box::new(stderr);
        self
    }

    /// Appends multiple environment variables at once for this builder.
    ///
    /// All environment variables are appended to the list of environment
    /// variables that this builder will configure.
    ///
    /// At this time environment variables are not deduplicated and if the same
    /// key is set twice then the guest will see two entries for the same key.
    ///
    /// # Examples
    ///
    /// ```
    /// use common_wasi::WasiCtxBuilder;
    ///
    /// let mut wasi = WasiCtxBuilder::new();
    /// wasi.envs(&[
    ///     ("FOO", "bar"),
    ///     ("HOME", "/somewhere"),
    /// ]);
    /// ```
    pub fn envs(&mut self, env: &[(impl AsRef<str>, impl AsRef<str>)]) -> &mut Self {
        self.env.extend(
            env.iter()
                .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned())),
        );
        self
    }

    /// Appends a single environment variable for this builder.
    ///
    /// At this time environment variables are not deduplicated and if the same
    /// key is set twice then the guest will see two entries for the same key.
    ///
    /// # Examples
    ///
    /// ```
    /// use common_wasi::WasiCtxBuilder;
    ///
    /// let mut wasi = WasiCtxBuilder::new();
    /// wasi.env("FOO", "bar");
    /// ```
    pub fn env(&mut self, k: impl AsRef<str>, v: impl AsRef<str>) -> &mut Self {
        self.env
            .push((k.as_ref().to_owned(), v.as_ref().to_owned()));
        self
    }

    /// Appends a list of arguments to the argument array to pass to wasm.
    pub fn args(&mut self, args: &[impl AsRef<str>]) -> &mut Self {
        self.args.extend(args.iter().map(|a| a.as_ref().to_owned()));
        self
    }

    /// Appends a single argument to get passed to wasm.
    pub fn arg(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Set the generator for the `wasi:random/random` number generator to the
    /// custom generator specified.
    ///
    /// Note that contexts have a default RNG configured which is a suitable
    /// generator for WASI and is configured with a random seed per-context.
    ///
    /// Guest code may rely on this random number generator to produce fresh
    /// unpredictable random data in order to maintain its security invariants,
    /// and ideally should use the insecure random API otherwise, so using any
    /// prerecorded or otherwise predictable data may compromise security.
    pub fn secure_random(&mut self, random: impl RngCore + Send + 'static) -> &mut Self {
        self.random = Box::new(random);
        self
    }

    /// Configures the generator for `wasi:random/insecure`.
    ///
    /// The `insecure_random` generator provided will be used for all randomness
    /// requested by the `wasi:random/insecure` interface.
    pub fn insecure_random(&mut self, insecure_random: impl RngCore + Send + 'static) -> &mut Self {
        self.insecure_random = Box::new(insecure_random);
        self
    }

    /// Configures the seed to be returned from `wasi:random/insecure-seed` to
    /// the specified custom value.
    ///
    /// By default this number is randomly generated when a builder is created.
    pub fn insecure_random_seed(&mut self, insecure_random_seed: u128) -> &mut Self {
        self.insecure_random_seed = insecure_random_seed;
        self
    }

    /// Configures `wasi:clocks/wall-clock` to use the `clock` specified.
    ///
    /// By default the host's wall clock is used.
    pub fn wall_clock(&mut self, clock: impl HostWallClock + 'static) -> &mut Self {
        self.wall_clock = Box::new(clock);
        self
    }

    /// Configures `wasi:clocks/monotonic-clock` to use the `clock` specified.
    ///
    /// By default the host's monotonic clock is used.
    pub fn monotonic_clock(&mut self, clock: impl HostMonotonicClock + 'static) -> &mut Self {
        self.monotonic_clock = Box::new(clock);
        self
    }

    /// Uses the configured context so far to construct the final [`WasiCtx`].
    ///
    /// Note that each `WasiCtxBuilder` can only be used to "build" once, and
    /// calling this method twice will panic.
    ///
    /// # Panics
    ///
    /// Panics if this method is called twice. Each [`WasiCtxBuilder`] can be
    /// used to create only a single [`WasiCtx`]. Repeated usage of this method
    /// is not allowed and should use a second builder instead.
    pub fn build(&mut self) -> WasiCtx {
        assert!(!self.built);

        let Self {
            stdin,
            stdout,
            stderr,
            env,
            args,
            random,
            insecure_random,
            insecure_random_seed,
            wall_clock,
            monotonic_clock,
            built: _,
        } = std::mem::replace(self, Self::new());
        self.built = true;

        WasiCtx {
            stdin,
            stdout,
            stderr,
            env,
            args,
            random,
            insecure_random,
            insecure_random_seed,
            wall_clock,
            monotonic_clock,
        }
    }
}

/// A trait which provides access to internal WASI state.
///
/// This trait is the basis of implementation of all traits in this crate. All
/// traits are implemented like:
///
/// ```
/// # trait WasiView {}
/// # mod bindings { pub mod wasi { pub trait Host {} } }
/// impl<T: WasiView> bindings::wasi::Host for T {
///     // ...
/// }
/// ```
///
/// For a [`Store<T>`](wasmtime::Store) this trait will be implemented
/// for the `T`. This also corresponds to the `T` in
/// [`Linker<T>`](wasmtime::component::Linker).
///
/// # Example
///
/// ```
/// use common_wasi::ResourceTable;
/// use common_wasi::{WasiCtx, WasiView, WasiCtxBuilder};
///
/// struct MyState {
///     ctx: WasiCtx,
///     table: ResourceTable,
/// }
///
/// impl WasiView for MyState {
///     fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
///     fn table(&mut self) -> &mut ResourceTable { &mut self.table }
/// }
///
/// impl MyState {
///     fn new() -> MyState {
///         let mut wasi = WasiCtxBuilder::new();
///         wasi.arg("./foo.wasm");
///         wasi.arg("--help");
///         wasi.env("FOO", "bar");
///
///         MyState {
///             ctx: wasi.build(),
///             table: ResourceTable::new(),
///         }
///     }
/// }
/// ```
pub trait WasiView: Send {
    /// Yields mutable access to the internal resource management that this
    /// context contains.
    ///
    /// Embedders can add custom resources to this table as well to give
    /// resources to wasm as well.
    fn table(&mut self) -> &mut ResourceTable;

    /// Yields mutable access to the configuration used for this context.
    ///
    /// The returned type is created through [`WasiCtxBuilder`].
    fn ctx(&mut self) -> &mut WasiCtx;
}

impl<T: ?Sized + WasiView> WasiView for &mut T {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(self)
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        T::ctx(self)
    }
}

impl<T: ?Sized + WasiView> WasiView for Box<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(self)
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        T::ctx(self)
    }
}

/// A small newtype wrapper which serves as the basis for implementations of
/// `Host` WASI traits in this crate.
///
/// This type is used as the basis for the implementation of all `Host` traits
/// generated by `bindgen!` for WASI interfaces. This is used automatically with
/// [`add_to_linker_sync`](crate::add_to_linker_sync) and
/// [`add_to_linker_async`](crate::add_to_linker_async).
///
/// This type is otherwise provided if you're calling the `add_to_linker`
/// functions generated by `bindgen!` from the [`bindings`
/// module](crate::bindings). In this situation you'll want to create a value of
/// this type in the closures added to a `Linker`.
#[repr(transparent)]
pub struct WasiImpl<T>(pub T);

impl<T: WasiView> WasiView for WasiImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0)
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        T::ctx(&mut self.0)
    }
}

/// Per-[`Store`] state which holds state necessary to implement WASI from this
/// crate.
///
/// This structure is created through [`WasiCtxBuilder`] and is stored within
/// the `T` of [`Store<T>`][`Store`]. Access to the structure is provided
/// through the [`WasiView`] trait as an implementation on `T`.
///
/// Note that this structure itself does not have any accessors, it's here for
/// internal use within the `wasmtime-wasi` crate's implementation of
/// bindgen-generated traits.
///
/// [`Store`]: wasmtime::Store
pub struct WasiCtx {
    pub(crate) random: Box<dyn RngCore + Send>,
    pub(crate) insecure_random: Box<dyn RngCore + Send>,
    pub(crate) insecure_random_seed: u128,
    pub(crate) wall_clock: Box<dyn HostWallClock + Send>,
    pub(crate) monotonic_clock: Box<dyn HostMonotonicClock + Send>,
    pub(crate) env: Vec<(String, String)>,
    pub(crate) args: Vec<String>,
    pub(crate) stdin: Box<dyn StdinStream>,
    pub(crate) stdout: Box<dyn StdoutStream>,
    pub(crate) stderr: Box<dyn StdoutStream>,
}

impl WasiCtx {
    /// Convenience function for calling [`WasiCtxBuilder::new`].
    pub fn builder() -> WasiCtxBuilder {
        WasiCtxBuilder::new()
    }
}
