use crate::language::{
    code::{
        Codebase, Expression, IntrinsicFunction, NodeKind, NodePath,
        SyntaxTree, Type,
    },
    packages::FunctionId,
};

use super::Value;

#[derive(Debug)]
pub struct Evaluator {
    next: Option<NodePath>,
    value: Value,
    effect: Option<Effect>,
}

impl Evaluator {
    pub fn new(codebase: &Codebase) -> Self {
        let next = Some(
            SyntaxTree::from_root(codebase.root().path)
                .find_leaf(codebase.nodes()),
        );

        Self {
            next,
            value: Value::None,
            effect: None,
        }
    }

    pub fn state<'r>(&self, codebase: &'r Codebase) -> EvaluatorState<'r> {
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
                EvaluatorState::Running {
                    expression,
                    path: _,
                } => {
                    break expression;
                }
                EvaluatorState::IgnoringEmptyFragment { path: _ } => {
                    self.advance(codebase);
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

    fn next<'r>(&self, codebase: &'r Codebase) -> EvaluatorState<'r> {
        let Some(path) = self.next else {
            return EvaluatorState::Finished { output: self.value };
        };

        if let Some(effect) = self.effect {
            return EvaluatorState::Effect { effect, path };
        }

        match &codebase.node_at(&path).kind {
            NodeKind::Empty => EvaluatorState::IgnoringEmptyFragment { path },
            NodeKind::Expression { expression } => {
                EvaluatorState::Running { expression, path }
            }
            NodeKind::Unresolved { name: _ } => EvaluatorState::Error { path },
        }
    }

    fn advance(&mut self, codebase: &Codebase) {
        self.next =
            self.next.as_ref().and_then(|next| codebase.parent_of(next));
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
        output: Value,
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
    Finished { output: Value },
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    ApplyHostFunction { id: FunctionId, input: Value },
    UnexpectedInput { expected: Type, actual: Value },
}
