/// A helper utility to provide [`std::fmt::Display`] for a [`Hash`]
/// to render as [`std::fmt::LowerHex`].
#[derive(PartialEq, Debug)]
pub struct HashDisplay(Vec<u8>);
impl std::fmt::Display for HashDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:x}", byte)?;
        }
        Ok(())
    }
}

impl From<Vec<u8>> for HashDisplay {
    fn from(value: Vec<u8>) -> Self {
        HashDisplay(value)
    }
}

/// Type of hash produced by an [`Encoder`].
pub type Hash = Vec<u8>;
/// Reference to a [`Hash`].
pub type HashRef = <Hash as std::ops::Deref>::Target;
