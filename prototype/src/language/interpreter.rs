use super::code::{Codebase, Expression, IntrinsicFunction, Location, Node};

#[derive(Debug)]
pub struct Interpreter {
    current_value: Value,
    next: Option<Location>,
}

impl Interpreter {
    pub fn new(codebase: &Codebase) -> Self {
        Self {
            current_value: Value::None,
            next: Some(codebase.entry()),
        }
    }

    pub fn next_step(&self) -> Option<&Location> {
        self.next.as_ref()
    }

    #[cfg(test)]
    pub fn provide_host_function_output(
        &mut self,
        value: Value,
        codebase: &Codebase,
    ) {
        // It would be nice to assert here, that a host function is actually
        // being applied. But we don't track that information currently.
        self.current_value = value;
        self.advance(codebase);
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let next = match self.next(codebase) {
            State::Running {
                expression,
                location,
            } => {
                let _ = location;
                expression
            }
            State::Finished => {
                return StepResult::Finished {
                    output: self.current_value,
                };
            }
            State::Error { location } => {
                let _ = location;
                return StepResult::Error;
            }
        };

        let value = match next {
            Expression::HostFunction { id } => {
                return StepResult::ApplyHostFunction {
                    id: *id,
                    input: self.current_value,
                };
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
        };

        self.current_value = value;
        self.advance(codebase);

        StepResult::FunctionApplied { output: value }
    }

    fn next<'r>(&mut self, codebase: &'r Codebase) -> State<'r> {
        loop {
            let Some(location) = self.next else {
                return State::Finished;
            };

            match codebase.node_at(&location) {
                Node::Empty => {
                    self.advance(codebase);
                    continue;
                }
                Node::Expression { expression } => {
                    return State::Running {
                        expression,
                        location,
                    };
                }
                Node::UnresolvedIdentifier { name: _ } => {
                    return State::Error { location };
                }
            }
        }
    }

    fn advance(&mut self, codebase: &Codebase) {
        self.next = self
            .next
            .as_ref()
            .and_then(|next| codebase.location_after(next));
    }
}

pub enum State<'r> {
    Running {
        expression: &'r Expression,
        location: Location,
    },
    Error {
        location: Location,
    },
    Finished,
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    FunctionApplied { output: Value },
    ApplyHostFunction { id: u32, input: Value },
    Finished { output: Value },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
