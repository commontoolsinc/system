use ct_common::{ConditionalSend, ConditionalSync};
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::{Node, Tree};

/// A key used to reference values in a [Tree] or [Node].
#[cfg(feature = "serde")]
pub trait Key:
    Serialize
    + for<'a> Deserialize<'a>
    + std::fmt::Debug
    + AsRef<[u8]>
    + ConditionalSync
    + ConditionalSend
    + Clone
    + PartialEq
    + Ord
{
}

#[cfg(not(feature = "serde"))]
pub trait Key:
    std::fmt::Debug + AsRef<[u8]> + ConditionalSync + ConditionalSend + Clone + PartialEq + Deref + Ord
{
}

impl Key for Vec<u8> {}
