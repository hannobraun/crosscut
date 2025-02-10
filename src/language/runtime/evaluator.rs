use crate::language::{
    code::{Codebase, Expression, IntrinsicFunction, NodeKind, NodePath, Type},
    packages::FunctionId,
};

use super::{Value, ValueWithSource};

#[derive(Debug)]
pub struct Evaluator {
    next: Vec<NodePath>,
    active_value: ValueWithSource,
    effect: Option<Effect>,
}

impl Evaluator {
    pub fn new(root: NodePath, codebase: &Codebase) -> Self {
        let mut evaluator = Self {
            next: Vec::new(),
            active_value: ValueWithSource {
                inner: Value::None,
                source: None,
            },
            effect: None,
        };

        evaluator.evaluate(root, codebase);

        evaluator
    }

    pub fn reset(&mut self, codebase: &Codebase) {
        *self = Self::new(codebase.root().path, codebase);
    }

    pub fn evaluate(&mut self, root: NodePath, codebase: &Codebase) {
        self.active_value = ValueWithSource {
            inner: Value::None,
            source: None,
        };
        let mut path = root;

        loop {
            self.next.push(path);

            if let NodeKind::Expression {
                expression:
                    Expression::IntrinsicFunction {
                        function:
                            IntrinsicFunction::Literal {
                                value: Value::Function { .. },
                            },
                    },
            } = codebase.node_at(&path).kind
            {
                // We have already pushed it, which means we're going to
                // evaluate it. But we need to stop here, since we don't want to
                // evaluate its body too, at least right here.
                break;
            }

            if let Some(child) = codebase.child_of(&path) {
                path = child;
                continue;
            } else {
                break;
            }
        }
    }

    pub fn state<'r>(&self, codebase: &'r Codebase) -> EvaluatorState<'r> {
        self.next(codebase)
    }

    pub fn provide_host_function_output(&mut self, value: Value) {
        let (Some(Effect::ApplyHostFunction { .. }), Some(source)) =
            (self.effect, self.next.last().copied())
        else {
            panic!(
                "Trying to provide host function output, but no host function \
                is currently being applied."
            );
        };

        self.effect = None;
        self.active_value = ValueWithSource {
            inner: value,
            source: Some(source),
        };
        self.advance();
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        self.effect = Some(effect);
    }

    pub fn step(&mut self, codebase: &Codebase) -> StepResult {
        let (next, path) = loop {
            match self.next(codebase) {
                EvaluatorState::Running { expression, path } => {
                    break (expression, path);
                }
                EvaluatorState::IgnoringEmptyFragment { path: _ } => {
                    self.advance();
                    continue;
                }
                EvaluatorState::Effect { effect, path: _ } => {
                    return StepResult::EffectTriggered { effect };
                }
                EvaluatorState::Error { path: _ } => {
                    return StepResult::Error;
                }
                EvaluatorState::Finished { output } => {
                    return StepResult::Finished { output };
                }
            }
        };

        self.active_value = match next {
            Expression::HostFunction { id } => {
                let effect = Effect::ApplyHostFunction {
                    id: *id,
                    input: self.active_value.inner,
                };
                self.effect = Some(effect);

                return StepResult::EffectTriggered { effect };
            }
            Expression::IntrinsicFunction { function } => {
                match function {
                    IntrinsicFunction::Identity => self.active_value,
                    IntrinsicFunction::Literal { value } => {
                        let Value::None = self.active_value.inner else {
                            // A literal is a function that takes `None`. If
                            // that isn't what we currently have, that's an
                            // error.
                            return StepResult::Error;
                        };

                        ValueWithSource {
                            inner: *value,
                            source: Some(path),
                        }
                    }
                }
            }
        };

        self.advance();

        StepResult::FunctionApplied {
            output: self.active_value.inner,
        }
    }

    fn next<'r>(&self, codebase: &'r Codebase) -> EvaluatorState<'r> {
        let Some(path) = self.next.last().copied() else {
            return EvaluatorState::Finished {
                output: self.active_value,
            };
        };

        if let Some(effect) = self.effect {
            return EvaluatorState::Effect { effect, path };
        }

        match &codebase.node_at(&path).kind {
            NodeKind::Empty => EvaluatorState::IgnoringEmptyFragment { path },
            NodeKind::Expression { expression } => {
                EvaluatorState::Running { expression, path }
            }
            NodeKind::Recursion => {
                todo!("Evaluating `self` is not supported yet.")
            }
            NodeKind::Error { node: _ } => EvaluatorState::Error { path },
        }
    }

    fn advance(&mut self) {
        self.next.pop();
    }
}

#[derive(Debug)]
pub enum EvaluatorState<'r> {
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
        output: ValueWithSource,
    },
}

impl EvaluatorState<'_> {
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
    Finished { output: ValueWithSource },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    ApplyHostFunction { id: FunctionId, input: Value },
    UnexpectedInput { expected: Type, actual: Value },
}
