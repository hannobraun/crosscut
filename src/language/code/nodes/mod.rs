mod children;
mod hash;
mod node;
mod nodes;

pub use self::{
    children::Children,
    hash::NodeHash,
    node::{Node, NodeKind},
    nodes::Nodes,
};

/// # A unique and versioned path to a [`Node`]
///
/// Builds on top of [`NodeHash`] to uniquely identify any syntax node.
///
/// [`NodePath`] is versioned, meaning that it will always point to the exact
/// same syntax node. If a newer version of that node exists, the same instance
/// of [`NodePath`] will still point to the original version.
///
/// ## Implementation Note
///
/// Right now, this struct only contains a [`NodeHash`], and is thus redundant.
/// But at some point, it will become possible to build identical expressions in
/// different parts of the syntax tree. That's when we're going to need this
/// struct.
///
/// And I want to already distinguish between this and [`NodeHash`] right now,
/// to make the API more clear, and to not require an eventual transition.
///
/// The specific change that will make this struct necessary, is supporting
/// syntax nodes with multiple children. (Thus turning the syntax tree into an
/// actual tree.) Once that is possible, we'll need two additional pieces of
/// data here, to uniquely identity a syntax node:
///
/// - The location of the parent node.
/// - The index of the child node, within the parent node's children.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodePath {
    /// # The hash of the node that this path uniquely identifies
    ///
    /// This hash isn't actually required to identify a node's position. The
    /// path to its parent and the index of the node within the parent's
    /// children, is actually enough to do that.
    ///
    /// But this hash actually is required to identify to identify the node
    /// _uniquely_, which includes the node's version.
    pub hash: NodeHash,
}

impl NodePath {
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
    ) -> impl Iterator<Item = LocatedNode<'r>> {
        self.node.children().into_iter().copied().map(move |hash| {
            let node = nodes.get(&hash);
            Self {
                node,
                path: NodePath { hash },
            }
        })
    }
}
