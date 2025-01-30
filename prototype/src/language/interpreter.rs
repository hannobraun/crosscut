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
        self.next = self
            .next
            .as_ref()
            .and_then(|next_step| codebase.location_after(next_step));
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let Some(next_step) = &self.next else {
            return StepResult::Finished {
                output: self.current_value,
            };
        };

        let next = codebase.node_at(next_step);

        let value = match next {
            Node::Empty => {
                // Empty nodes are ignored during execution. Those are only
                // added by the editor as a placeholder.
                self.current_value
            }
            Node::Expression { expression } => {
                match expression {
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
                }
            }
            Node::UnresolvedIdentifier { name: _ } => {
                return StepResult::Error;
            }
        };

        self.current_value = value;
        self.next = codebase.location_after(next_step);

        StepResult::FunctionApplied { output: value }
    }
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
