use std::fmt;

use crate::language::{code::Expression, packages::Packages};

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

    #[cfg(test)]
    pub fn integer_literal(value: i32) -> Self {
        Self::new(NodeKind::integer_literal(value), None)
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

    pub fn to_token(&self, packages: &Packages) -> String {
        self.display(packages).to_string()
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
        use crate::language::code::Literal;

        Self::Expression {
            expression: Expression::Literal {
                literal: Literal::Integer { value },
            },
        }
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
