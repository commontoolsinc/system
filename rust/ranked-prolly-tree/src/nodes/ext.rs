use crate::{Entry, Error, Node, Result, Storage};
use async_trait::async_trait;
use nonempty::NonEmpty;

/// Additional [`Node`] functionality for debugging and rendering.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait NodeExt<const P: u8> {
    /// Return all entries from this node into a [`Entry`] collection.
    ///
    /// Returns an error if this is not a segment node.
    fn into_entries(self) -> Result<NonEmpty<Entry>>;

    /// Decode all children refs for this node from `storage` into a [`Node`] collection.
    ///
    /// Returns an error is this is not a branch node.
    async fn into_children(self, storage: &impl Storage) -> Result<NonEmpty<Node<P>>>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<const P: u8> NodeExt<P> for Node<P> {
    fn into_entries(self) -> Result<NonEmpty<Entry>> {
        if !self.is_segment() {
            return Err(Error::SegmentOnly);
        }

        self.block.into_entries()
    }

    async fn into_children(self, storage: &impl Storage) -> Result<NonEmpty<Node<P>>> {
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
