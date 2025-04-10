use super::{Node, NodeHash, Nodes};

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
    /// I initially considered this to be the right trade-off, but I've had a
    /// new idea since then: Use a hash here, let's call it `ParentHash`, that
    /// includes both the parent's `NodeHash` and an `Option<ParentHash>`, for
    /// the grandparent (which would then recursively include the whole
    /// lineage).
    ///
    /// This would remove the need for heap allocation here, as well as allow
    /// [`NodePath`] to be `Copy` again. On the other hand, it would make it
    /// more complicated to find the parent of a node, given its `NodePath`.
    parent: Option<Box<NodePath>>,

    /// # The index of the node among its siblings
    sibling_index: SiblingIndex,
}

impl NodePath {
    #[track_caller]
    pub fn new(
        hash: NodeHash,
        parent: Option<NodePath>,
        sibling_index: SiblingIndex,
        nodes: &Nodes,
    ) -> Self {
        if let Some(parent) = &parent {
            if nodes
                .get(&parent.hash)
                .has_child(&hash, &sibling_index)
                .is_none()
            {
                panic!(
                    "Attempting to construct invalid `NodePath`: Node is not \
                    listed among children of its supposed parent.",
                );
            }
        }

        let parent = parent.map(Box::new);
        Self {
            hash,
            parent,
            sibling_index,
        }
    }

    pub fn for_root(hash: NodeHash) -> Self {
        Self {
            hash,
            parent: None,
            sibling_index: SiblingIndex { index: 0 },
        }
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
    pub fn parent(&self) -> Option<&NodePath> {
        self.parent.as_deref()
    }

    pub fn sibling_index(&self) -> SiblingIndex {
        self.sibling_index
    }

    pub fn is_ancestor_of(&self, possible_descendant: &NodePath) -> bool {
        let mut maybe_parent = possible_descendant.parent.as_deref();

        while let Some(parent) = maybe_parent {
            if parent == self {
                return true;
            }

            maybe_parent = parent.parent.as_deref();
        }

        false
    }

    #[cfg(test)]
    pub fn distance_from_root(&self) -> u32 {
        let mut distance_from_root = 0;
        let mut parent = self.parent();

        while let Some(p) = parent {
            distance_from_root += 1;
            parent = p.parent();
        }

        distance_from_root
    }
}

/// # The index of a node among its siblings
///
/// ## Implementation Note
///
/// I'm a bit concerned with the use of `usize` here, as it could lead to
/// problems when serializing `Codebase`. But using something else makes some
/// other code much harder to write. I'd basically have to re-implement
/// `iter::Enumerate`, including its implementation of `DoubleEndedIterator, for
/// `u32` or whatever.
///
/// For now, this works. But it might have to change going forward.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct SiblingIndex {
    pub index: usize,
}

impl SiblingIndex {
    pub fn dec(self) -> Self {
        let Some(index) = self.index.checked_sub(1) else {
            panic!("Can't decrement sibling index `{}`", self.index);
        };

        Self { index }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<'r> {
    pub node: &'r Node,
    pub path: NodePath,
}

impl<'r> LocatedNode<'r> {
    pub fn children(
        &self,
        nodes: &'r Nodes,
    ) -> impl DoubleEndedIterator<Item = LocatedNode<'r>> {
        self.node.to_children().into_iter().enumerate().map(
            move |(index, hash)| {
                let node = nodes.get(&hash);
                Self {
                    node,
                    path: NodePath::new(
                        hash,
                        Some(self.path.clone()),
                        SiblingIndex { index },
                        nodes,
                    ),
                }
            },
        )
    }
}
