use super::code::{Codebase, Expression, Node};

#[derive(Debug)]
pub struct Interpreter {
    current_value: Value,
}

impl Interpreter {
    pub fn new(_: &Codebase) -> Self {
        Self {
            current_value: Value::None,
        }
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let value = match codebase.nodes.first() {
            Some(Node::Empty) => {
                // Empty nodes are ignored during execution. Those are only
                // added by the editor as a placeholder.
                self.current_value
            }
            Some(Node::Expression {
                expression: Expression::LiteralValue { value: output },
            }) => {
                let Value::None = self.current_value else {
                    // A literal is a function that takes `None`. If that isn't
                    // what we currently have, that's an error.
                    return StepResult::Error;
                };

                *output
            }
            None => self.current_value,
        };

        self.current_value = value;

        StepResult::Finished { output: value }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    Finished { output: Value },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
