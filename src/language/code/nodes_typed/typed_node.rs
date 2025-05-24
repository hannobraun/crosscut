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

            SyntaxNode::Apply { .. }
            | SyntaxNode::Body { .. }
            | SyntaxNode::Empty
            | SyntaxNode::Function { .. }
            | SyntaxNode::Identifier { .. }
            | SyntaxNode::Number { .. }
            | SyntaxNode::Recursion
            | SyntaxNode::Tuple { .. } => Self::Expression,

            SyntaxNode::Binding { .. } => Self::Pattern,
        }
    }
}
