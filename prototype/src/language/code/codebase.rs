use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub nodes: Vec<Node>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: Node) -> Location {
        let location = Location {
            index: self.nodes.len(),
        };
        self.nodes.push(node);
        location
    }

    pub fn replace(&mut self, to_replace: &Location, replacement: Node) {
        self.nodes[to_replace.index] = replacement;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Expression { expression: Expression },
}

#[derive(Debug)]
pub struct Location {
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    IntrinsicFunction { function: IntrinsicFunction },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntrinsicFunction {
    LiteralValue { value: Value },
}
