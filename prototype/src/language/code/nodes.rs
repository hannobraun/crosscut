use std::fmt;

use crate::language::host::Host;

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Node {
    Empty,
    Expression { expression: Expression },
    Unresolved { name: String },
}

impl Node {
    pub fn display<'r>(&'r self, host: &'r Host) -> NodeDisplay<'r> {
        NodeDisplay { node: self, host }
    }
}

pub struct NodeDisplay<'r> {
    node: &'r Node,
    host: &'r Host,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node {
            Node::Empty => {
                write!(f, "")
            }
            Node::Expression { expression } => {
                write!(f, "{}", expression.display(self.host))
            }
            Node::Unresolved { name } => {
                write!(f, "{name}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeId {
    hash: [u8; 32],
}

impl NodeId {
    pub fn generate_for(node: &Node) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(node).into();
        Self { hash }
    }
}
