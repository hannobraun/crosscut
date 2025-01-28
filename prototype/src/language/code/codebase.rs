use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub value: Value,
}

impl Codebase {
    pub fn new() -> Self {
        Self { value: Value::None }
    }
}
