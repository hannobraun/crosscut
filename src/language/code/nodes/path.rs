use super::{Node, NodeHash, Nodes};

/// # A unique and versioned path to a [`Node`]
///
/// Builds on top of [`NodeHash`] to uniquely identify any syntax node within
/// the syntax tree.
///
/// [`NodePath`] is versioned, meaning that it will always point to the exact
/// same syntax node. If a newer version of that node exists, the same instance
/// of [`NodePath`] will still point to the original version.
///
/// ## Implementation Note
///
/// At this point, [`NodePath`] can't distinguish between identical siblings of
/// the same parent. To do that, the index of the node withing the parent's
/// children needs to be added here.
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
    ///
    /// ## Implementation Note
    ///
    /// I'm a bit concerned with using `usize` here, as it could lead to
    /// problems when serializing `Codebase`. But using something else makes
    /// some other code much harder to write. I'd basically have to re-implement
    /// `iter::Enumerate`, including its implementation of `DoubleEndedIterator,
    /// for `u32` or whatever.
    ///
    /// For now, this works. But it might have to change going forward.
    sibling_index: usize,
}

impl NodePath {
    pub fn new(
        hash: NodeHash,
        parent: Option<NodePath>,
        sibling_index: usize,
        nodes: &Nodes,
    ) -> Self {
        if let Some(parent) = &parent {
            assert!(
                nodes.get(&parent.hash).has_this_child(&hash),
                "Attempting to construct invalid `NodePath`: Node being \
                referred to is not among its purported children.",
            );
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
            sibling_index: 0,
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

    pub fn sibling_index(&self) -> usize {
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
                        index,
                        nodes,
                    ),
                }
            },
        )
    }
}
