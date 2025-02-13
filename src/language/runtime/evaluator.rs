use crate::language::{
    code::{
        Codebase, Expression, IntrinsicFunction, Literal, NodeKind, NodePath,
        Type,
    },
    packages::FunctionId,
};

use super::{Value, ValueWithSource};

#[derive(Debug)]
pub struct Evaluator {
    root: NodePath,
    next: Vec<NodePath>,
    active_value: Vec<ValueWithSource>,
    effect: Option<Effect>,
}

impl Evaluator {
    pub fn new(root: NodePath, codebase: &Codebase) -> Self {
        let mut evaluator = Self {
            root,
            next: Vec::new(),
            active_value: Vec::new(),
            effect: None,
        };

        evaluator.evaluate(evaluator.root, Value::Nothing, codebase);

        evaluator
    }

    pub fn reset(&mut self, codebase: &Codebase) {
        *self = Self::new(codebase.root().path, codebase);
    }

    pub fn evaluate(
        &mut self,
        root: NodePath,
        active_value: Value,
        codebase: &Codebase,
    ) {
        self.root = root;
        self.active_value.push(ValueWithSource {
            inner: active_value,
            source: None,
        });
        let mut path = root;

        loop {
            self.next.push(path);

            if let NodeKind::Expression {
                expression:
                    Expression::IntrinsicFunction {
                        intrinsic:
                            IntrinsicFunction::Literal {
                                literal: Literal::Function,
                            },
                    },
            } = codebase.node_at(&path).kind
            {
                // We have already pushed the function literal, which means
                // we're going to evaluate it. But we need to stop here, since
                // we don't want to evaluate the function itself right now.
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
            (self.effect.clone(), self.next.last().copied())
        else {
            panic!(
                "Trying to provide host function output, but no host function \
                is currently being applied."
            );
        };

        self.effect = None;
        *self.active_value.last_mut().unwrap() = ValueWithSource {
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
                EvaluatorState::IgnoringSyntaxNode => {
                    self.advance();
                    continue;
                }
                EvaluatorState::Recursing => {
                    self.evaluate(self.root, Value::Nothing, codebase);

                    // We could `continue` here. Then the following call to
                    // `Self::next` above would return the next expression we
                    // need to evaluate, and we could immediately do that.
                    // Without bothering the caller about this recursion, which
                    // would become an internal implementation detail.
                    //
                    // But that won't work, because of one very important edge
                    // case: If `self.root` points to nothing except a bare
                    // `self` without any children, then we would immediately
                    // land back here, producing an endless loop and hanging the
                    // caller.
                    //
                    // And endless loop that does nothing is likely a problem
                    // either way, but it's not our responsibility to address
                    // that. All we're doing here is evaluate Crosscut code, so
                    // let's do that, and let the caller decide what to do about
                    // endless loops in that.
                    return StepResult::Recursing;
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

        let active_value = self.active_value.last_mut().unwrap();

        match next {
            Expression::HostFunction { id } => {
                let effect = Effect::ApplyHostFunction {
                    id: *id,
                    input: active_value.inner.clone(),
                };
                self.effect = Some(effect.clone());

                return StepResult::EffectTriggered { effect };
            }
            Expression::IntrinsicFunction { intrinsic } => {
                match intrinsic {
                    IntrinsicFunction::Identity => {
                        // Active value stays the same.
                    }
                    IntrinsicFunction::Literal { literal } => {
                        let Value::Nothing = active_value.inner else {
                            // A literal is a function that takes `None`. If
                            // that isn't what we currently have, that's an
                            // error.

                            // The compiler doesn't know about this error. If we
                            // want the return value of `state` to reflect it,
                            // we need to keep track of it here.
                            self.effect = Some(Effect::UnexpectedInput {
                                expected: Type::Nothing,
                                actual: active_value.inner.clone(),
                            });

                            return StepResult::Error;
                        };

                        let value = {
                            match *literal {
                                Literal::Function => {
                                    let Some(child) = codebase.child_of(&path)
                                    else {
                                        unreachable!(
                                            "Function literal must have a \
                                            child, or it wouldn't have been \
                                            resolved as a function literal."
                                        );
                                    };

                                    Value::Function {
                                        body: *child.hash(),
                                    }
                                }
                                Literal::Integer { value } => {
                                    Value::Integer { value }
                                }
                                Literal::Tuple => {
                                    // Evaluating tuples is not supported yet.
                                    return StepResult::Error;
                                }
                            }
                        };

                        *active_value = ValueWithSource {
                            inner: value,
                            source: Some(path),
                        };
                    }
                }
            }
        };

        let result = StepResult::FunctionApplied {
            output: active_value.inner.clone(),
        };

        self.advance();

        result
    }

    fn next<'r>(&self, codebase: &'r Codebase) -> EvaluatorState<'r> {
        let Some(path) = self.next.last().copied() else {
            let output =
                self.active_value
                    .last()
                    .cloned()
                    .unwrap_or(ValueWithSource {
                        inner: Value::Nothing,
                        source: None,
                    });

            return EvaluatorState::Finished { output };
        };

        if let Some(effect) = self.effect.clone() {
            return EvaluatorState::Effect { effect, path };
        }

        match &codebase.node_at(&path).kind {
            NodeKind::Empty => EvaluatorState::IgnoringSyntaxNode,
            NodeKind::Expression { expression } => {
                EvaluatorState::Running { expression, path }
            }
            NodeKind::Recursion => EvaluatorState::Recursing,
            NodeKind::Error { node: _ } => EvaluatorState::Error { path },
        }
    }

    fn advance(&mut self) {
        self.next.pop();
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EvaluatorState<'r> {
    Running {
        expression: &'r Expression,
        path: NodePath,
    },
    IgnoringSyntaxNode,
    Recursing,
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
            Self::IgnoringSyntaxNode => None,
            Self::Recursing => None,
            Self::Effect { effect: _, path } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { output: _ } => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    FunctionApplied { output: Value },
    Recursing,
    EffectTriggered { effect: Effect },
    Finished { output: ValueWithSource },
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    ApplyHostFunction { id: FunctionId, input: Value },
    UnexpectedInput { expected: Type, actual: Value },
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Codebase, Node, NodeKind},
        runtime::{Evaluator, StepResult},
    };

    #[test]
    fn handle_bare_recursion() {
        // Recursion can quite naturally be implemented in a way that results in
        // an endless loop within `step`, if the evaluated expression consists
        // of nothing but a `self`. And in fact, that's what the first draft
        // did.

        let mut codebase = Codebase::new();
        codebase.insert_as_parent_of(
            codebase.root().path,
            Node {
                kind: NodeKind::Recursion,
                child: Some(*codebase.root().path.hash()),
            },
        );

        let mut evaluator = Evaluator::new(codebase.root().path, &codebase);

        assert_eq!(evaluator.step(&codebase), StepResult::Recursing);
    }
}
