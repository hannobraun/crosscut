use std::{collections::BTreeMap, fmt};

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

use crate::language::host::Host;

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
        let hash = NodeHash {
            hash: udigest::hash::<blake3::Hasher>(&node).into(),
        };
        self.inner.insert(hash, node);
        hash
    }
}

/// The hash of a syntax node
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

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Node {
    pub kind: NodeKind,

    /// The syntax node that provides the input for this one
    ///
    /// Can be `None`, if this is a leaf node that has no children.
    ///
    /// Including this field ensures, that [`NodeId`] is computed for the whole
    /// syntax sub-tree, for which this syntax node is the root. Otherwise,
    /// syntax nodes could share the same ID with other nodes that are not
    /// identical.
    ///
    /// For example, all applications of a given function would have the same
    /// ID, regardless of their arguments.
    pub child: Option<NodeHash>,
}

impl Node {
    pub fn empty(child: Option<NodeHash>) -> Self {
        Self {
            child,
            kind: NodeKind::Empty,
        }
    }

    pub fn display<'r>(&'r self, host: &'r Host) -> NodeDisplay<'r> {
        NodeDisplay { node: self, host }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum NodeKind {
    Empty,
    Expression { expression: Expression },
    Unresolved { name: String },
}

pub struct NodeDisplay<'r> {
    node: &'r Node,
    host: &'r Host,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.node.kind {
            NodeKind::Empty => {
                write!(f, "")
            }
            NodeKind::Expression { expression } => {
                write!(f, "{}", expression.display(self.host))
            }
            NodeKind::Unresolved { name } => {
                write!(f, "{name}")
            }
        }
    }
}

/// A unique and versioned path to a [`Node`]
///
/// Builds on top of [`NodeHash`] to uniquely identify any syntax node.
///
/// [`NodePath`] is versioned, meaning that it will always point to the exact
/// same syntax node. If a newer version of that node exists, the same instance
/// of [`NodePath`] will still point to the original version.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodePath {
    pub(super) hash: NodeHash,
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
