use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub value: Expression,
}

impl Codebase {
    pub fn new() -> Self {
        Self {
            value: Expression::LiteralValue { value: Value::None },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    LiteralValue { value: Value },
}
