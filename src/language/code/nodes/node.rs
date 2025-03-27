use std::{fmt, option, slice};

use crate::language::packages::{FunctionId, Packages};

use super::{Children, NodeHash};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Node {
    kind: NodeKind,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn has_this_child(&self, child: &NodeHash) -> bool {
        match &self.kind {
            NodeKind::Empty { child: children } => {
                children.as_ref() == Some(child)
            }
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralInteger { children, .. }
            | NodeKind::LiteralTuple { children }
            | NodeKind::ProvidedFunction { children, .. }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => {
                children.inner.contains(child)
            }
        }
    }

    pub fn has_no_children(&self) -> bool {
        match &self.kind {
            NodeKind::Empty { child: children } => children.is_none(),
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralInteger { children, .. }
            | NodeKind::LiteralTuple { children }
            | NodeKind::ProvidedFunction { children, .. }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => children.is_empty(),
        }
    }

    pub fn has_single_child(&self) -> Option<&NodeHash> {
        match &self.kind {
            NodeKind::Empty { child: children } => children.as_ref(),
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralInteger { children, .. }
            | NodeKind::LiteralTuple { children }
            | NodeKind::ProvidedFunction { children, .. }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => children.is_single_child(),
        }
    }

    pub fn children(&self) -> ChildrenIter {
        match &self.kind {
            NodeKind::Empty { child: children } => ChildrenIter::Option {
                iter: children.iter(),
            },
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralInteger { children, .. }
            | NodeKind::LiteralTuple { children }
            | NodeKind::ProvidedFunction { children, .. }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => ChildrenIter::Slice {
                iter: children.iter(),
            },
        }
    }

    pub fn to_children(&self) -> Children {
        match &self.kind {
            NodeKind::Empty { child: children } => Children::new(*children),
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralInteger { children, .. }
            | NodeKind::LiteralTuple { children }
            | NodeKind::ProvidedFunction { children, .. }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => children.clone(),
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

pub enum ChildrenIter<'r> {
    Option { iter: option::Iter<'r, NodeHash> },
    Slice { iter: slice::Iter<'r, NodeHash> },
}

impl<'r> Iterator for ChildrenIter<'r> {
    type Item = &'r NodeHash;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Option { iter } => iter.next(),
            Self::Slice { iter } => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Option { iter } => iter.size_hint(),
            Self::Slice { iter } => iter.size_hint(),
        }
    }
}

impl DoubleEndedIterator for ChildrenIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Option { iter } => iter.next_back(),
            Self::Slice { iter } => iter.next_back(),
        }
    }
}

impl ExactSizeIterator for ChildrenIter<'_> {}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum NodeKind {
    Empty { child: Option<NodeHash> },
    LiteralFunction { children: Children },
    LiteralInteger { value: i32, children: Children },
    LiteralTuple { children: Children },
    ProvidedFunction { id: FunctionId, children: Children },
    Recursion { children: Children },
    Error { node: String, children: Children },
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
