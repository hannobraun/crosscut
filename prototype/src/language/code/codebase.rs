use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub value: Vec<Expression>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { value: Vec::new() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    LiteralValue { value: Value },
}
