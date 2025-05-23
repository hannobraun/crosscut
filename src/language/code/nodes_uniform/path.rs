use std::ops::Deref;

use super::{ChildIndex, NodeHash, Nodes};

/// # A unique and versioned path to a [`Node`]
///
/// Builds on top of [`NodeHash`] to uniquely identify any syntax node within
/// the syntax tree.
///
/// ## Attention: Any Change to the Syntax Tree May Invalidate a [`NodePath`]
///
/// [`NodePath`] contains a copy of its parent's [`NodePath`]. This is the main
/// thing that distinguishes it from [`NodeHash`]: a [`NodeHash`]'s contents
/// only depends on the node it refers to and its children. It won't be able to
/// distinguish between identical nodes or subtrees that are located in
/// different places.
///
/// But a [`NodePath`] can do that, because it depends on its parent's
/// [`NodePath`]. And that parent path will depend on its parent, recursively,
/// all the way to the root. And the root is going to change if _any_ node in
/// the syntax tree changes.
///
/// That means **any [`NodePath`] that you expect to point to a node within the
/// current syntax tree will be invalidated any change to the syntax tree**. You
/// are responsible for making sure that such a [`NodePath`] gets updated.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodePath {
    hash: NodeHash,

    /// # The path of the node's parent
    ///
    /// ## Implementation Note
    ///
    /// The `Box` is required here for indirection, but is both potentially
    /// expensive (in terms of performance, due to memory allocations) and
    /// inconvenient (as it prevents this type from being `Copy`).
    ///
    /// It would also be possible to use a hash here,let's call it
    /// `ParentHash`, that includes both the parent's `NodeHash` and an
    /// `Option<ParentHash>`, for the grandparent (which would then recursively
    /// include the whole lineage).
    ///
    /// This would remove the need for heap allocation here, as well as allow
    /// [`NodePath`] to be `Copy` again. But it would make it more complicated
    /// to find the parent of a node, given its `NodePath`, and we'd need much
    /// more bookkeeping in `Codebase` or `Nodes` to compensate.
    ///
    /// When I tried this approach, it didn't seem worth the trouble.
    parent: Option<(Box<NodePath>, ChildIndex)>,
}

impl NodePath {
    #[track_caller]
    pub fn new(
        hash: NodeHash,
        parent: Option<(NodePath, ChildIndex)>,
        nodes: &Nodes,
    ) -> Self {
        if let Some((parent_path, index)) = &parent {
            let parent_node = nodes.get(parent_path.hash());

            if !parent_node.children().contains(&hash, index) {
                panic!(
                    "Attempting to construct invalid `NodePath`: Node is not \
                    listed among children of its supposed parent, at the given \
                    sibling index.\n\
                    \n\
                    Trying to construct `NodePath` for hash `{hash:?}` with \
                    sibling index {index}.\n\
                    \n\
                    Parent:\n\
                    \n\
                    {parent_node:#?}\n\
                    \n\
                    At path:\n\
                    \n\
                    {parent_path:#?}",
                );
            }
        }

        Self {
            hash,
            parent: parent.map(|(path, index)| (Box::new(path), index)),
        }
    }

    pub fn for_root(hash: NodeHash) -> Self {
        Self { hash, parent: None }
    }

    /// # The hash of the node that this path uniquely identifies
    ///
    /// This hash isn't actually required to identify a node's position. The
    /// path to its parent and the index of the node within the parent's
    /// children would actually be enough to do that.
    ///
    /// But this hash is required to identify the node _uniquely_, including its
    /// version.
    pub fn hash(&self) -> &NodeHash {
        &self.hash
    }

    /// # The path of the node's parent
    ///
    /// This is required to distinguish between identical nodes whose hash is
    /// the same, but that have different parents.
    pub fn parent(&self) -> Option<(&NodePath, ChildIndex)> {
        self.parent
            .as_ref()
            .map(|(path, index)| (path.deref(), *index))
    }

    pub fn is_ancestor_of(&self, possible_descendant: &NodePath) -> bool {
        let mut maybe_parent = possible_descendant.parent.as_ref();

        while let Some((parent, _)) = maybe_parent {
            if parent.deref() == self {
                return true;
            }

            maybe_parent = parent.parent.as_ref();
        }

        false
    }
}
