/// Host-provided features that augment the runtime.
pub trait HostFeatures {
    /// Callback executed when runtime invokes
    /// `callback` in `common:basic/host-callback@0.0.1`.
    fn host_callback(input: String) -> std::result::Result<String, String> {
        Ok(input)
    }
}
