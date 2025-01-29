use super::code::{Codebase, Expression, IntrinsicFunction, Location, Node};

#[derive(Debug)]
pub struct Interpreter {
    current_value: Value,
    next_step: Option<Location>,
}

impl Interpreter {
    pub fn new(codebase: &Codebase) -> Self {
        Self {
            current_value: Value::None,
            next_step: Some(codebase.entry()),
        }
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let Some(next_step) = &self.next_step else {
            return StepResult::Finished {
                output: self.current_value,
            };
        };

        let next = codebase.node_at(next_step);
        self.next_step = codebase.location_after(next_step);

        let value = match next {
            Node::Empty => {
                // Empty nodes are ignored during execution. Those are only
                // added by the editor as a placeholder.
                self.current_value
            }
            Node::Expression { expression } => {
                match expression {
                    Expression::HostFunction { id: _ } => {
                        todo!("Host functions are not supported yet.");
                    }
                    Expression::IntrinsicFunction { function } => {
                        match function {
                            IntrinsicFunction::Identity => self.current_value,
                            IntrinsicFunction::Literal { value } => {
                                let Value::None = self.current_value else {
                                    // A literal is a function that takes
                                    // `None`. If that isn't what we currently
                                    // have, that's an error.
                                    return StepResult::Error;
                                };

                                *value
                            }
                        }
                    }
                }
            }
        };

        self.current_value = value;

        StepResult::Application { output: value }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    Application { output: Value },
    Finished { output: Value },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
