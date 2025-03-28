use std::fmt;

use crate::language::packages::{FunctionId, Packages};

use super::{Children, NodeHash};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Node {
    Empty {
        child: Option<NodeHash>,
    },

    LiteralFunction {
        body: NodeHash,
    },

    LiteralInteger {
        value: i32,
    },

    LiteralTuple {
        children: Children,
    },

    ProvidedFunction {
        id: FunctionId,
        child: Option<NodeHash>,
    },

    Recursion {
        child: Option<NodeHash>,
    },

    Error {
        node: String,
        children: Children,
    },
}

impl Node {
    pub fn has_this_child(&self, child: &NodeHash) -> bool {
        match self {
            Self::Empty { child: c }
            | Self::ProvidedFunction { child: c, .. }
            | Self::Recursion { child: c } => c.as_ref() == Some(child),
            Self::LiteralInteger { value: _ } => false,
            Self::LiteralFunction { body } => body == child,
            Self::LiteralTuple { children } | Self::Error { children, .. } => {
                children.inner.contains(child)
            }
        }
    }

    pub fn has_no_children(&self) -> bool {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction { child, .. }
            | Self::Recursion { child } => child.is_none(),
            Self::LiteralInteger { value: _ } => true,
            Self::LiteralFunction {
                body: NodeHash { .. },
            } => false,
            Self::LiteralTuple { children } | Self::Error { children, .. } => {
                children.is_empty()
            }
        }
    }

    pub fn has_single_child(&self) -> Option<&NodeHash> {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction { child, .. }
            | Self::Recursion { child } => child.as_ref(),
            Self::LiteralInteger { value: _ } => None,
            Self::LiteralFunction { body } => Some(body),
            Self::LiteralTuple { children } | Self::Error { children, .. } => {
                children.is_single_child()
            }
        }
    }

    pub fn to_children(&self) -> Children {
        match self {
            Self::Empty { child }
            | Self::ProvidedFunction { child, .. }
            | Self::Recursion { child } => Children::new(*child),
            Self::LiteralInteger { value: _ } => Children::new([]),
            Self::LiteralFunction { body } => Children::new([*body]),
            Self::LiteralTuple { children } | Self::Error { children, .. } => {
                children.clone()
            }
        }
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

pub struct NodeDisplay<'r> {
    node: &'r Node,
    packages: &'r Packages,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.node {
            Node::Empty { .. } => {
                write!(f, "")
            }
            Node::LiteralFunction { .. } => {
                write!(f, "fn")
            }
            Node::LiteralInteger { value } => {
                write!(f, "{value}")
            }
            Node::LiteralTuple { .. } => {
                write!(f, "tuple")
            }
            Node::ProvidedFunction { id, .. } => {
                let name = self.packages.function_name_by_id(id);
                write!(f, "{name}")
            }
            Node::Recursion { .. } => {
                write!(f, "self")
            }
            Node::Error { node, .. } => {
                write!(f, "{node}")
            }
        }
    }
}
