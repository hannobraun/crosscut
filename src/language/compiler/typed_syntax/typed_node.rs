use crate::language::code::{NodeHash, SiblingIndex, SyntaxNode};

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

    pub fn replace_child(
        &mut self,
        replace_hash: &NodeHash,
        replace_index: &SiblingIndex,
        replacement: NodeHash,
    ) -> bool {
        match self {
            TypedNode::Expression { expression } => match expression {
                Expression::Apply { apply } => apply.replace_child(
                    replace_hash,
                    replace_index,
                    replacement,
                ),
                Expression::Function { function } => function.replace_child(
                    replace_hash,
                    replace_index,
                    replacement,
                ),
                Expression::Tuple { tuple } => tuple.replace_child(
                    replace_hash,
                    replace_index,
                    replacement,
                ),
                Expression::Other => false,
            },
            TypedNode::Pattern | TypedNode::Other => false,
        }
    }

    pub fn into_syntax_node(self) -> SyntaxNode {
        match self {
            TypedNode::Expression { expression } => match expression {
                Expression::Apply { apply } => apply.into_syntax_node(),
                Expression::Function { function } => {
                    function.into_syntax_node()
                }
                Expression::Tuple { tuple } => tuple.into_syntax_node(),
                Expression::Other => {
                    panic!("Can't convert node into syntax node.");
                }
            },
            TypedNode::Pattern | TypedNode::Other => {
                panic!("Can't convert node into syntax node.");
            }
        }
    }
}

pub enum Expression {
    Apply { apply: Apply<NodeByHash> },
    Function { function: Function<NodeByHash> },
    Tuple { tuple: Tuple<NodeByHash> },

    Other,
}
