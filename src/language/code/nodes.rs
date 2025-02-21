use std::{collections::BTreeMap, fmt, slice, vec};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};

use crate::language::packages::Packages;

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nodes {
    inner: BTreeMap<NodeHash, Node>,
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, hash: &NodeHash) -> &Node {
        let Some(node) = self.inner.get(hash) else {
            unreachable!(
                "This is an append-only data structure. All hashes that were \
                ever created must be valid."
            );
        };

        node
    }

    pub fn insert(&mut self, node: Node) -> NodeHash {
        let hash = NodeHash::new(&node);
        self.inner.insert(hash, node);
        hash
    }
}

/// # The hash of a syntax node
///
/// The purpose of this type is to serve as a building block for identifying
/// syntax nodes in a unique and versioned manner. But it's important to
/// understand that it is not more than a building block. By itself, a hash
/// can't be unique.
///
/// A hash derives from a syntax node's contents, which include its children.
/// This is not unique, because the syntax tree can contain identical sub-trees
/// in various places, each with a root node that would have identical hashes.
///
/// To guarantee uniqueness, we also need the position of the node within the
/// syntax tree, which includes the parent. But a node's parent already includes
/// its own children, which would cause a circular dependency when computing the
/// hash.
///
/// To solve this issue, we have [`NodePath`], which after the hashes have been
/// constructed bottom-up, is constructed top-down, from the root of the syntax
/// tree.
///
/// ## Implementation Note
///
/// Not everything documented here, especially the relation to [`NodePath`], is
/// implemented yet. I wanted to paint a complete picture of how this fits into
/// my current plans, even though those are not fully realized yet.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct NodeHash {
    hash: [u8; 32],
}

impl NodeHash {
    fn new(node: &Node) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(&node).into();
        Self { hash }
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NodeHash")
            .field("hash", &BASE64_URL_SAFE_NO_PAD.encode(self.hash))
            .finish()
    }
}

impl fmt::Display for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_URL_SAFE_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}

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
        use super::IntrinsicFunction;
        use crate::language::code::Literal;

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
    pub child: Vec<NodeHash>,
}

impl Children {
    pub fn new(children: impl IntoIterator<Item = NodeHash>) -> Self {
        let mut children = children.into_iter();
        let child = children.next();

        assert!(
            children.next().is_none(),
            "Syntax nodes with multiple children are not fully supported yet.",
        );

        Self {
            child: child.into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.child.is_empty()
    }

    pub fn contains(&self, child: &NodeHash) -> bool {
        self.child.contains(child)
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn has_one(&self) -> Option<&NodeHash> {
        assert!(
            self.child.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );

        self.child.first()
    }

    pub fn add(&mut self, to_add: NodeHash) {
        assert!(
            self.child.is_empty(),
            "Syntax nodes with multiple children are not fully supported yet.",
        );

        self.child.push(to_add);
    }

    pub fn replace(
        &mut self,
        to_replace: &NodeHash,
        replacements: impl IntoIterator<Item = NodeHash>,
    ) {
        let mut replacements = replacements.into_iter();

        assert!(
            self.child.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );
        assert_eq!(
            self.child.first(),
            Some(to_replace),
            "Trying to replace child that is not present.",
        );

        self.child.clear();
        self.child.extend(replacements.next());

        assert!(
            replacements.next().is_none(),
            "Replacing a child with multiple other children is not supported \
            yet.",
        );
    }

    pub fn to_paths(&self) -> impl Iterator<Item = NodePath> {
        self.child.iter().copied().map(|hash| NodePath { hash })
    }
}

impl IntoIterator for Children {
    type Item = NodeHash;
    type IntoIter = vec::IntoIter<NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.child.into_iter()
    }
}

impl<'r> IntoIterator for &'r Children {
    type Item = &'r NodeHash;
    type IntoIter = slice::Iter<'r, NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.child.iter()
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
