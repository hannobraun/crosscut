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
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodePath {
    pub hash: NodeHash,

    /// # The path of the node's parent
    ///
    /// This is required to distinguish between identical nodes whose hash is
    /// the same, but that have different parents.
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
    /// more complicated to find the parent of a node, given its `NodePath`.s
    pub parent: Option<Box<NodePath>>,
}

impl NodePath {
    pub fn new(hash: NodeHash, parent: Option<NodePath>) -> Self {
        let parent = parent.map(Box::new);
        Self { hash, parent }
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
        self.node.children().iter().copied().map(move |hash| {
            let node = nodes.get(&hash);
            Self {
                node,
                path: NodePath::new(hash, Some(self.path.clone())),
            }
        })
    }
}
