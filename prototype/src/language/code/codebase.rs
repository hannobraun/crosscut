use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub nodes: Vec<Node>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn replace(&mut self, replacement: Node) {
        self.nodes = vec![replacement];
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Expression { expression: Expression },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    LiteralValue { value: Value },
}
