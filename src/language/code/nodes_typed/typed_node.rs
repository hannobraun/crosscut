use crate::language::code::{Nodes, SyntaxNode};

pub enum TypedNode {
    Expression,
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_syntax_node(syntax_node: &SyntaxNode, _: &Nodes) -> Self {
        match syntax_node {
            SyntaxNode::Add => Self::Other,
            SyntaxNode::Apply { .. } => Self::Expression,
            SyntaxNode::Binding { .. } => Self::Pattern,
            SyntaxNode::Body { .. } => Self::Expression,
            SyntaxNode::Empty => Self::Expression,
            SyntaxNode::Function { .. } => Self::Expression,
            SyntaxNode::Identifier { .. } => Self::Expression,
            SyntaxNode::Number { .. } => Self::Expression,
            SyntaxNode::Recursion => Self::Expression,
            SyntaxNode::Tuple { .. } => Self::Expression,
        }
    }
}
