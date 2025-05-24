use crate::language::code::SyntaxNode;

pub enum TypedNode {
    Expression,
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_syntax_node(syntax_node: &SyntaxNode) -> Self {
        match syntax_node {
            SyntaxNode::Add => Self::Other,
            SyntaxNode::Apply { .. } => Self::Expression,
            SyntaxNode::Body { .. } => Self::Expression,
            SyntaxNode::Empty => Self::Expression,
            SyntaxNode::Function { .. } => Self::Expression,
            SyntaxNode::Identifier { .. } => Self::Expression,
            SyntaxNode::Number { .. } => Self::Expression,
            SyntaxNode::Recursion => Self::Expression,
            SyntaxNode::Tuple { .. } => Self::Expression,
            SyntaxNode::Binding { .. } => Self::Pattern,
        }
    }
}
