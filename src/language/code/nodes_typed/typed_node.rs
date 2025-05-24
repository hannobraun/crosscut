use crate::{
    language::code::{NodeByHash, NodeHash, Nodes, SyntaxNode},
    util::form::Owned,
};

use super::{Apply, Body, Function, Tuple};

#[derive(Debug)]
pub enum TypedNode {
    Expression { expression: Expression },
    Pattern,
    Other,
}

impl TypedNode {
    pub fn from_hash(hash: &NodeHash, nodes: &Nodes) -> Self {
        let syntax_node = nodes.get(hash);
        Self::from_syntax_node(syntax_node, nodes)
    }

    pub fn from_syntax_node(syntax_node: &SyntaxNode, nodes: &Nodes) -> Self {
        match syntax_node.clone() {
            SyntaxNode::Add => Self::Other,
            SyntaxNode::Apply {
                expression,
                argument,
            } => Self::Expression {
                expression: Expression::Apply {
                    apply: Apply {
                        expression,
                        argument,
                    },
                },
            },
            SyntaxNode::Binding { .. } => Self::Pattern,
            SyntaxNode::Body { children, add } => Self::Expression {
                expression: Expression::Body {
                    body: Body { children, add },
                },
            },
            SyntaxNode::Empty => Self::Expression {
                expression: Expression::Empty,
            },
            SyntaxNode::Function { parameter, body } => Self::Expression {
                expression: Expression::Function {
                    function: Function::new(&parameter, body, nodes),
                },
            },
            SyntaxNode::Identifier { name } => Self::Expression {
                expression: Expression::Identifier { name },
            },
            SyntaxNode::Number { value } => Self::Expression {
                expression: Expression::Number { value },
            },
            SyntaxNode::Recursion => Self::Expression {
                expression: Expression::Recursion,
            },
            SyntaxNode::Tuple { values, add_value } => Self::Expression {
                expression: Expression::Tuple {
                    tuple: Tuple { values, add_value },
                },
            },
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Apply { apply: Apply<NodeByHash> },
    Body { body: Body<NodeByHash> },
    Empty,
    Function { function: Function<Owned> },
    Identifier { name: String },
    Number { value: i32 },
    Recursion,
    Tuple { tuple: Tuple<NodeByHash> },
}
