use ct_common::{ConditionalSend, ConditionalSync};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[cfg(doc)]
use crate::{Node, Tree};

/// A key used to reference values in a [Tree] or [Node].
pub trait Key:
    AsRef<[u8]>
    + Serialize
    + for<'a> Deserialize<'a>
    + ConditionalSync
    + ConditionalSend
    + Clone
    + PartialEq
    + Deref
    + Ord
{
}

impl Key for Vec<u8> {}
