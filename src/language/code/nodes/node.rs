use std::fmt;

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
            NodeKind::Empty { child: c }
            | NodeKind::ProvidedFunction { children: c, .. } => {
                c.as_ref() == Some(child)
            }
            NodeKind::LiteralInteger { value: _ } => false,
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralTuple { children }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => {
                children.inner.contains(child)
            }
        }
    }

    pub fn has_no_children(&self) -> bool {
        match &self.kind {
            NodeKind::Empty { child }
            | NodeKind::ProvidedFunction {
                children: child, ..
            } => child.is_none(),
            NodeKind::LiteralInteger { value: _ } => true,
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralTuple { children }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => children.is_empty(),
        }
    }

    pub fn has_single_child(&self) -> Option<&NodeHash> {
        match &self.kind {
            NodeKind::Empty { child }
            | NodeKind::ProvidedFunction {
                children: child, ..
            } => child.as_ref(),
            NodeKind::LiteralInteger { value: _ } => None,
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralTuple { children }
            | NodeKind::Recursion { children }
            | NodeKind::Error { children, .. } => children.is_single_child(),
        }
    }

    pub fn to_children(&self) -> Children {
        match &self.kind {
            NodeKind::Empty { child }
            | NodeKind::ProvidedFunction {
                children: child, ..
            } => Children::new(*child),
            NodeKind::LiteralInteger { value: _ } => Children::new([]),
            NodeKind::LiteralFunction { children }
            | NodeKind::LiteralTuple { children }
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

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum NodeKind {
    Empty {
        child: Option<NodeHash>,
    },

    LiteralFunction {
        /// # The children of the function
        ///
        /// ## Implementation Note
        ///
        /// Currently, it's generally assumed that a function has one child, its
        /// body. (Although this isn't checked much, or at all.) In contrast to
        /// the other variants of this enum, I've decided not to enforce that
        /// via the type of this field though.
        ///
        /// That would complicate the compiler, which would need to check the
        /// number of children when constructing this variant. And that wouldn't
        /// be worth it, because soon, functions will have two children (they
        /// need parameters), and eventually probably an arbitrary number (any
        /// number of branches).
        ///
        /// I'd rather see this shake out, before making changes here that would
        /// only be made invalid.
        children: Children,
    },

    LiteralInteger {
        value: i32,
    },

    LiteralTuple {
        children: Children,
    },

    ProvidedFunction {
        id: FunctionId,
        children: Option<NodeHash>,
    },

    Recursion {
        children: Children,
    },

    Error {
        node: String,
        children: Children,
    },
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
