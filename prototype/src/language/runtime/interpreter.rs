use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, NodeKind, NodePath, Type,
};

use super::Value;

#[derive(Debug)]
pub struct Interpreter {
    next: Option<NodePath>,
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
        let Some(Effect::ApplyHostFunction { .. }) = self.effect else {
            panic!(
                "Trying to provide host function output, but no host function \
                is currently being applied."
            );
        };

        // It would be nice to assert here, that a host function is actually
        // being applied. But we don't track that information currently.
        self.effect = None;
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
                    path: _,
                } => {
                    break expression;
                }
                InterpreterState::IgnoringEmptyFragment { path: _ } => {
                    self.advance(codebase);
                    continue;
                }
                InterpreterState::Effect { effect, path: _ } => {
                    return StepResult::EffectTriggered { effect };
                }
                InterpreterState::Error { path: _ } => {
                    return StepResult::Error;
                }
                InterpreterState::Finished { output } => {
                    return StepResult::Finished { output };
                }
            }
        };

        self.value = match next {
            Expression::HostFunction { id } => {
                let effect = Effect::ApplyHostFunction {
                    id: *id,
                    input: self.value,
                };
                self.effect = Some(effect);

                return StepResult::EffectTriggered { effect };
            }
            Expression::IntrinsicFunction { function } => {
                match function {
                    IntrinsicFunction::Identity => self.value,
                    IntrinsicFunction::Literal { value } => {
                        let Value::None = self.value else {
                            // A literal is a function that takes `None`. If
                            // that isn't what we currently have, that's an
                            // error.
                            return StepResult::Error;
                        };

                        *value
                    }
                }
            }
        };

        self.advance(codebase);

        StepResult::FunctionApplied { output: self.value }
    }

    fn next<'r>(&self, codebase: &'r Codebase) -> InterpreterState<'r> {
        let Some(path) = self.next else {
            return InterpreterState::Finished { output: self.value };
        };

        if let Some(effect) = self.effect {
            return InterpreterState::Effect { effect, path };
        }

        match &codebase.node_at(&path).kind {
            NodeKind::Empty => InterpreterState::IgnoringEmptyFragment { path },
            NodeKind::Expression { expression } => {
                InterpreterState::Running { expression, path }
            }
            NodeKind::Unresolved { name: _ } => {
                InterpreterState::Error { path }
            }
        }
    }

    fn advance(&mut self, codebase: &Codebase) {
        self.next =
            self.next.as_ref().and_then(|next| codebase.parent_of(next));
    }
}

#[derive(Debug)]
pub enum InterpreterState<'r> {
    Running {
        expression: &'r Expression,
        path: NodePath,
    },
    IgnoringEmptyFragment {
        path: NodePath,
    },
    Effect {
        effect: Effect,
        path: NodePath,
    },
    Error {
        path: NodePath,
    },
    Finished {
        output: Value,
    },
}

impl InterpreterState<'_> {
    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Running {
                expression: _,
                path,
            } => Some(path),
            Self::IgnoringEmptyFragment { path } => Some(path),
            Self::Effect { effect: _, path } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { output: _ } => None,
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
    UnexpectedInput { expected: Type, actual: Value },
}
