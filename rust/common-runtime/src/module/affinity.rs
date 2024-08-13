/// Defines the intended proximity of Module instantiation relative to the host device
/// that is currently running a Runtime instance.
#[derive(Clone)]
pub enum Affinity {
    /// Only instantiate *on* the local device
    LocalOnly,
    /// Only instantiate *off* of the local device
    RemoteOnly,
    /// Instantiate on or off the local device, but prefer on the local device if
    /// possible
    PrefersLocal,
    /// Instantiate on or off the local device, but prefer off the local device
    /// if possible
    PrefersRemote,
}
