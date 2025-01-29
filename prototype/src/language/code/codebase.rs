use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub nodes: Vec<Node>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Expression { expression: Expression },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    LiteralValue { value: Value },
}
