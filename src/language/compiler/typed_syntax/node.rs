use crate::language::code::SyntaxNode;

pub enum TypedNode {
    Expression { expression: Expression },
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_syntax_node(syntax_node: &SyntaxNode) -> Self {
        match syntax_node {
            SyntaxNode::AddNode => Self::Other,

            SyntaxNode::Apply { .. } => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Binding { .. } => Self::Pattern,
            SyntaxNode::Empty => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Function { .. } => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Identifier { .. } => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Number { .. } => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Recursion => Self::Expression {
                expression: Expression,
            },
            SyntaxNode::Tuple { .. } => Self::Expression {
                expression: Expression,
            },
        }
    }
}

pub struct Expression;
