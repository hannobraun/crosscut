use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub expressions: Vec<Expression>,
}

impl Codebase {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    LiteralValue { value: Value },
}
