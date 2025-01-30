use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Location, Node, Type,
};

#[derive(Debug)]
pub struct Interpreter {
    next: Option<Location>,
    value: Value,
    effect: Option<Effect>,
}

impl Interpreter {
    pub fn new(codebase: &Codebase) -> Self {
        Self {
            next: Some(codebase.entry()),
            value: Value::None,
            effect: None,
        }
    }

    pub fn state<'r>(&self, codebase: &'r Codebase) -> InterpreterState<'r> {
        self.next(codebase)
    }

    pub fn provide_host_function_output(
        &mut self,
        value: Value,
        codebase: &Codebase,
    ) {
        // It would be nice to assert here, that a host function is actually
        // being applied. But we don't track that information currently.
        self.value = value;
        self.advance(codebase);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.effect = Some(effect);
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let next = loop {
            match self.next(codebase) {
                InterpreterState::Running {
                    expression,
                    location: _,
                } => {
                    break expression;
                }
                InterpreterState::IgnoringEmptyFragment { location: _ } => {
                    self.advance(codebase);
                    continue;
                }
                InterpreterState::Effect {
                    effect,
                    location: _,
                } => {
                    return StepResult::EffectTriggered { effect };
                }
                InterpreterState::Error { location: _ } => {
                    return StepResult::Error;
                }
                InterpreterState::Finished => {
                    return StepResult::Finished { output: self.value };
                }
            }
        };

        let value = match next {
            Expression::HostFunction { id } => {
                return StepResult::EffectTriggered {
                    effect: Effect::ApplyHostFunction {
                        id: *id,
                        input: self.value,
                    },
                };
            }
            Expression::IntrinsicFunction { function } => {
                match function {
                    IntrinsicFunction::Identity => self.value,
                    IntrinsicFunction::Literal { value } => {
                        let Value::None = self.value else {
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

        self.value = value;
        self.advance(codebase);

        StepResult::FunctionApplied { output: value }
    }

    fn next<'r>(&self, codebase: &'r Codebase) -> InterpreterState<'r> {
        let Some(location) = self.next else {
            return InterpreterState::Finished;
        };

        if let Some(effect) = self.effect {
            return InterpreterState::Effect { effect, location };
        }

        match codebase.node_at(&location) {
            Node::Empty => InterpreterState::IgnoringEmptyFragment { location },
            Node::Expression { expression } => InterpreterState::Running {
                expression,
                location,
            },
            Node::UnresolvedIdentifier { name: _ } => {
                InterpreterState::Error { location }
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

pub enum InterpreterState<'r> {
    Running {
        expression: &'r Expression,
        location: Location,
    },
    IgnoringEmptyFragment {
        location: Location,
    },
    Effect {
        effect: Effect,
        location: Location,
    },
    Error {
        location: Location,
    },
    Finished,
}

impl InterpreterState<'_> {
    pub fn location(&self) -> Option<&Location> {
        match self {
            Self::Running {
                expression: _,
                location,
            } => Some(location),
            Self::IgnoringEmptyFragment { location } => Some(location),
            Self::Effect {
                effect: _,
                location,
            } => Some(location),
            Self::Error { location } => Some(location),
            Self::Finished => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    FunctionApplied { output: Value },
    EffectTriggered { effect: Effect },
    Finished { output: Value },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    ApplyHostFunction { id: u32, input: Value },
    UnexpectedInput { expected: Type },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
