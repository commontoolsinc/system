use common_protos::runtime;

/// The mode with which to attempt to instantiate a Common Module. The
/// [InstantiationMode] should be taken as a stated preference; it is observed
/// on a best-effort basis, and may be substituted for a different mode if the
/// preferred one is unavailable. Rust cannot be interpreted. It is possible that
/// some languages may not be able to be compiled.
pub enum InstantiationMode {
    /// Attempt to interpret the module; this only works for modules with
    /// available sources in a language that supports runtime interpretation
    /// e.g., JavaScript and Python
    Interpret,
    /// Compile and run the module as a stand-alone Wasm Component, or fetch and
    /// run such a Wasm Component if it is already compiled. Sources must be
    /// available if the component is not already available in a compiled form.
    Compile,
}

impl From<runtime::InstantiationMode> for InstantiationMode {
    fn from(value: runtime::InstantiationMode) -> Self {
        match value {
            runtime::InstantiationMode::Compile => InstantiationMode::Compile,
            runtime::InstantiationMode::Interpret => InstantiationMode::Interpret,
        }
    }
}

impl From<InstantiationMode> for runtime::InstantiationMode {
    fn from(value: InstantiationMode) -> Self {
        match value {
            InstantiationMode::Compile => runtime::InstantiationMode::Compile,
            InstantiationMode::Interpret => runtime::InstantiationMode::Interpret,
        }
    }
}
