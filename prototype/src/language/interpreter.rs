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
            Next::Node { expression: node } => node,
            Next::NoMoreNodes => {
                return StepResult::Finished {
                    output: self.current_value,
                };
            }
            Next::Error => {
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

    fn next<'r>(&mut self, codebase: &'r Codebase) -> Next<'r> {
        let expression = loop {
            let Some(next) = self.next else {
                return Next::NoMoreNodes;
            };

            match codebase.node_at(&next) {
                Node::Empty => {
                    self.advance(codebase);
                    continue;
                }
                Node::Expression { expression } => {
                    break expression;
                }
                Node::UnresolvedIdentifier { name: _ } => {
                    return Next::Error;
                }
            }
        };

        Next::Node { expression }
    }

    fn advance(&mut self, codebase: &Codebase) {
        self.next = self
            .next
            .as_ref()
            .and_then(|next| codebase.location_after(next));
    }
}

enum Next<'r> {
    Node { expression: &'r Expression },
    NoMoreNodes,
    Error,
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
