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

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Node {
    pub kind: NodeKind,

    /// The syntax node that provides the input for this one
    ///
    /// Can be `None`, if this is the first syntax node in its context.
    ///
    /// Including this field ensures, that [`NodeId`] is computed for the whole
    /// syntax sub-tree, for which this syntax node is the root. Otherwise,
    /// syntax nodes could share the same ID with other nodes.
    ///
    /// For example, all applications of a given function would have the same
    /// ID, despite differing arguments.
    pub input: Option<NodeHash>,
}

impl Node {
    pub fn empty(input: Option<NodeHash>) -> Self {
        Self {
            input,
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Location {
    pub(super) index: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocatedNode<'r> {
    pub node: &'r Node,
    pub location: Location,
}
