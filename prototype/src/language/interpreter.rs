use super::code::{Codebase, Expression};

#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let Expression::LiteralValue { value: output } = codebase
            .expressions
            .first()
            .cloned()
            .unwrap_or(Expression::LiteralValue { value: Value::None });
        StepResult::Finished { output }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    Finished { output: Value },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
