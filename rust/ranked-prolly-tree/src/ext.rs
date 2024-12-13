use crate::{Error, Key, Node, Result, Storage};
use async_trait::async_trait;
use nonempty::NonEmpty;

/// Additional [`Node`] functionality for debugging and rendering.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait NodeExt<const P: u8, K: Key> {
    /// Decode all children refs for this node from `storage` into a [`Node`] collection.
    ///
    /// Returns an error is this is not a branch node.
    async fn into_children(self, storage: &impl Storage<K>) -> Result<NonEmpty<Node<P, K>>>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8, K> NodeExt<P, K> for Node<P, K>
where
    K: Key + 'static,
{
    async fn into_children(self, storage: &impl Storage<K>) -> Result<NonEmpty<Node<P, K>>> {
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
