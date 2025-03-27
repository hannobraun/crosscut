use std::{fmt, slice};

use crate::language::packages::{FunctionId, Packages};

use super::{Children, NodeHash};

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

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn has_this_child(&self, child: &NodeHash) -> bool {
        self.children.inner.contains(child)
    }

    pub fn has_no_children(&self) -> bool {
        self.children.is_empty()
    }

    pub fn has_single_child(&self) -> Option<&NodeHash> {
        self.children.is_single_child()
    }

    pub fn children(&self) -> slice::Iter<NodeHash> {
        self.children.iter()
    }

    pub fn to_children(&self) -> Children {
        self.children.clone()
    }

    pub fn to_token(&self, packages: &Packages) -> String {
        self.display(packages).to_string()
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
    LiteralFunction,
    LiteralInteger { value: i32 },
    LiteralTuple,
    ProvidedFunction { id: FunctionId },
    Recursion,
    Error { node: String },
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
            NodeKind::LiteralFunction { .. } => {
                write!(f, "fn")
            }
            NodeKind::LiteralInteger { value, .. } => {
                write!(f, "{value}")
            }
            NodeKind::LiteralTuple { .. } => {
                write!(f, "tuple")
            }
            NodeKind::ProvidedFunction { id, .. } => {
                let name = self.packages.function_name_by_id(id);
                write!(f, "{name}")
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
