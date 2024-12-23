use crate::{Error, Key, Node, Result, Storage};
use async_trait::async_trait;
use ct_common::ConditionalSync;
use nonempty::NonEmpty;

/// Additional [`Node`] functionality for debugging and rendering.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait NodeExt<const P: u8, K: Key, V: ConditionalSync> {
    /// Decode all children refs for this node from `storage` into a [`Node`] collection.
    ///
    /// Returns an error is this is not a branch node.
    async fn into_children<S: Storage<K, V>>(self, storage: &S) -> Result<NonEmpty<Node<P, K, V>>>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, K, V> NodeExt<P, K, V> for Node<P, K, V>
where
    K: Key + 'static,
    V: Clone + ConditionalSync,
{
    async fn into_children<S: Storage<K, V>>(self, storage: &S) -> Result<NonEmpty<Node<P, K, V>>> {
        if !self.is_branch() {
            return Err(Error::BranchOnly);
        }
        let mut output = vec![];
        for node_ref in self.block.into_node_refs()? {
            output.push(Node::from_ref(node_ref, storage).await?);
        }
        NonEmpty::from_vec(output).ok_or(Error::Unexpected)
    }
}
