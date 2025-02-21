pub mod hash;
pub mod nodes;

pub use self::{hash::NodeHash, nodes::Nodes};

use std::{fmt, slice, vec};

use crate::language::packages::Packages;

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Node {
    kind: NodeKind,
    children: Children,
}

impl Node {
    pub fn new(
        kind: NodeKind,
        children: impl IntoIterator<Item = NodeHash>,
    ) -> Self {
        let children = Children::new(children);
        Self { kind, children }
    }

    #[cfg(test)]
    pub fn integer_literal(value: i32, child: Option<NodeHash>) -> Self {
        Self::new(NodeKind::integer_literal(value), child)
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn children(&self) -> &Children {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Children {
        &mut self.children
    }

    pub fn display<'r>(&'r self, packages: &'r Packages) -> NodeDisplay<'r> {
        NodeDisplay {
            node: self,
            packages,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum NodeKind {
    Empty,
    Expression { expression: Expression },
    Recursion,
    Error { node: String },
}

impl NodeKind {
    #[cfg(test)]
    pub fn integer_literal(value: i32) -> Self {
        use crate::language::code::{IntrinsicFunction, Literal};

        Self::Expression {
            expression: Expression::IntrinsicFunction {
                intrinsic: IntrinsicFunction::Literal {
                    literal: Literal::Integer { value },
                },
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Children {
    children: Vec<NodeHash>,
}

impl Children {
    pub fn new(children: impl IntoIterator<Item = NodeHash>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn contains(&self, child: &NodeHash) -> bool {
        self.children.contains(child)
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn has_one(&self) -> Option<&NodeHash> {
        assert!(
            self.children.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );

        self.children.first()
    }

    pub fn add(&mut self, to_add: NodeHash) {
        assert!(
            self.children.is_empty(),
            "Syntax nodes with multiple children are not fully supported yet.",
        );

        self.children.push(to_add);
    }

    pub fn replace(
        &mut self,
        to_replace: &NodeHash,
        replacements: impl IntoIterator<Item = NodeHash>,
    ) {
        let mut replacements = replacements.into_iter();

        assert!(
            self.children.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );
        assert_eq!(
            self.children.first(),
            Some(to_replace),
            "Trying to replace child that is not present.",
        );

        self.children.clear();
        self.children.extend(replacements.next());

        assert!(
            replacements.next().is_none(),
            "Replacing a child with multiple other children is not supported \
            yet.",
        );
    }

    pub fn to_paths(&self) -> impl Iterator<Item = NodePath> {
        self.children.iter().copied().map(|hash| NodePath { hash })
    }
}

impl IntoIterator for Children {
    type Item = NodeHash;
    type IntoIter = vec::IntoIter<NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl<'r> IntoIterator for &'r Children {
    type Item = &'r NodeHash;
    type IntoIter = slice::Iter<'r, NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

pub struct NodeDisplay<'r> {
    node: &'r Node,
    packages: &'r Packages,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.node.kind {
            NodeKind::Empty { .. } => {
                write!(f, "")
            }
            NodeKind::Expression { expression, .. } => {
                write!(f, "{}", expression.display(self.packages))
            }
            NodeKind::Recursion { .. } => {
                write!(f, "self")
            }
            NodeKind::Error { node, .. } => {
                write!(f, "{node}")
            }
        }
    }
}

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
