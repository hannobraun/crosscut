use super::code::{Codebase, Expression, Node};

#[derive(Debug)]
pub struct Interpreter {
    current_value: Value,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            current_value: Value::None,
        }
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let Node::Expression {
            expression: Expression::LiteralValue { value: output },
        } = codebase.nodes.first().cloned().unwrap_or(Node::Expression {
            expression: Expression::LiteralValue {
                value: self.current_value,
            },
        });
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
