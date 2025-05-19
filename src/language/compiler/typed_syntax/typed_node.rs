use crate::language::code::SyntaxNode;

use super::{Apply, Function, NodeByHash, Tuple};

pub enum TypedNode {
    Expression { expression: Expression },
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_syntax_node(syntax_node: &SyntaxNode) -> Self {
        match syntax_node {
            SyntaxNode::AddNode => Self::Other,
            SyntaxNode::Apply {
                expression,
                argument,
            } => Self::Expression {
                expression: Expression::Apply {
                    apply: Apply {
                        expression: *expression,
                        argument: *argument,
                    },
                },
            },
            SyntaxNode::Binding { .. } => Self::Pattern,
            SyntaxNode::Empty => Self::Expression {
                expression: Expression::Other,
            },
            SyntaxNode::Function { parameter, body } => Self::Expression {
                expression: Expression::Function {
                    function: Function {
                        parameter: *parameter,
                        body: body.clone(),
                    },
                },
            },
            SyntaxNode::Identifier { .. } => Self::Expression {
                expression: Expression::Other,
            },
            SyntaxNode::Number { .. } => Self::Expression {
                expression: Expression::Other,
            },
            SyntaxNode::Recursion => Self::Expression {
                expression: Expression::Other,
            },
            SyntaxNode::Tuple { values, add_value } => Self::Expression {
                expression: Expression::Tuple {
                    tuple: Tuple {
                        values: values.clone(),
                        add_value: *add_value,
                    },
                },
            },
        }
    }
}

pub enum Expression {
    Apply { apply: Apply<NodeByHash> },
    Function { function: Function<NodeByHash> },
    Tuple { tuple: Tuple<NodeByHash> },

    Other,
}
