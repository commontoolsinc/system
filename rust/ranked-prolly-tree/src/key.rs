#[cfg(doc)]
use crate::{Node, Tree};
use ct_common::{ConditionalSend, ConditionalSync};

/// A key used to reference values in a [Tree] or [Node].
pub trait Key:
    std::fmt::Debug + AsRef<[u8]> + ConditionalSync + ConditionalSend + Clone + PartialEq + Ord
{
}

impl Key for Vec<u8> {}
