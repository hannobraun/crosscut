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
    contexts: Vec<Context>,
    state: EvaluatorState,
}

impl Evaluator {
    pub fn new(root: NodePath, codebase: &Codebase) -> Self {
        let mut evaluator = Self {
            root,
            contexts: Vec::new(),
            state: EvaluatorState::Running {
                active_value: ValueWithSource {
                    inner: Value::Nothing,
                    source: None,
                },
            },
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
        let mut nodes_from_root = Vec::new();
        let mut path = root;

        loop {
            nodes_from_root.push(path);

            if let NodeKind::Expression {
                expression:
                    Expression::IntrinsicFunction {
                        intrinsic:
                            IntrinsicFunction::Literal {
                                literal: Literal::Function | Literal::Tuple,
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

        let active_value = ValueWithSource {
            inner: active_value,
            source: None,
        };
        self.state = EvaluatorState::Running {
            active_value: active_value.clone(),
        };
        self.contexts.push(Context {
            nodes_from_root,
            active_value,
        });
    }

    pub fn provide_host_function_output(&mut self, value: Value) {
        let EvaluatorState::Effect {
            effect: Effect::ApplyHostFunction { .. },
            ..
        } = &self.state
        else {
            panic!(
                "Trying to provide host function output, but no host function \
                is currently being applied."
            );
        };
        let Some(context) = self.contexts.last_mut() else {
            unreachable!(
                "Host function is being applied, but no context is available. \
                This should not be possible, because without a context, what \
                would have triggered the effect?"
            );
        };
        let Some(source) = context.nodes_from_root.last().copied() else {
            unreachable!(
                "Host function is being applied, but there doesn't seem to be \
                a syntax node that could have triggered it."
            );
        };

        context.active_value = ValueWithSource {
            inner: value,
            source: Some(source),
        };
        self.state = EvaluatorState::Running {
            active_value: context.active_value.clone(),
        };

        self.advance();
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        let Some(context) = self.contexts.last_mut() else {
            panic!(
                "Not allowed to trigger effect, if there is no context it \
                could come from."
            );
        };
        let Some(path) = context.nodes_from_root.last().copied() else {
            panic!(
                "Not allowed to trigger effect, if there is no active syntax \
                node that could trigger it."
            );
        };

        self.state = EvaluatorState::Effect { effect, path };
    }

    pub fn step(&mut self, codebase: &Codebase) {
        let (expression, path) = loop {
            match self.next(codebase) {
                Next::Running { expression, path } => {
                    break (expression, path);
                }
                Next::IgnoringSyntaxNode => {
                    self.advance();
                    continue;
                }
                Next::Recursing => {
                    self.evaluate(self.root, Value::Nothing, codebase);

                    // We could `continue` here. Then the next call to
                    // `Self::next` above would return the next expression we
                    // need to evaluate, and we could immediately do that.
                    // Without bothering the caller with making an otherwise
                    // useless step.
                    //
                    // But that won't work, because of one very important edge
                    // case: If `self.root` points to nothing except a bare
                    // `self` without any children, then we would immediately
                    // land back here, producing an endless loop and hanging the
                    // caller.
                    //
                    // An endless loop that does nothing is likely a problem
                    // either way, but it's not our responsibility to address
                    // that. All we're doing here is evaluate Crosscut code, so
                    // let's do that, and let the caller decide what to do about
                    // endless loops.
                    return;
                }
                Next::Effect { effect, path } => {
                    self.state = EvaluatorState::Effect { effect, path };
                    return;
                }
                Next::Error { path } => {
                    self.state = EvaluatorState::Error { path };
                    return;
                }
                Next::Finished { output } => {
                    self.state = EvaluatorState::Finished { output };
                    return;
                }
            }
        };

        // It would be nicer, if `next` could return the context to us. It must
        // have had one available, or we wouldn't be here right now.
        //
        // But in addition to making the lifetimes more complicated, this would
        // require `next` to take `&mut self`. Which wouldn't be a problem for
        // the use here, but `next` is also called from `state`, which doesn't
        // have (and shouldn't need!) `&mut self`.
        let Some(context) = self.contexts.last_mut() else {
            unreachable!(
                "A context must be available, or `next` wouldn't have returned \
                `EvaluatorState::Running`, and this wouldn't get executed."
            );
        };

        match expression {
            Expression::HostFunction { id } => {
                self.state = EvaluatorState::Effect {
                    effect: Effect::ApplyHostFunction {
                        id: *id,
                        input: context.active_value.inner.clone(),
                    },
                    path,
                };

                return;
            }
            Expression::IntrinsicFunction { intrinsic } => {
                match intrinsic {
                    IntrinsicFunction::Identity => {
                        // Active value stays the same.
                    }
                    IntrinsicFunction::Literal { literal } => {
                        let Value::Nothing = context.active_value.inner else {
                            // A literal is a function that takes `None`. If
                            // that isn't what we currently have, that's an
                            // error.

                            // The compiler doesn't know about this error. If we
                            // want the return value of `state` to reflect it,
                            // we need to keep track of it here.
                            self.state = EvaluatorState::Effect {
                                effect: Effect::UnexpectedInput {
                                    expected: Type::Nothing,
                                    actual: context.active_value.inner.clone(),
                                },
                                path,
                            };

                            return;
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
                                    self.state = EvaluatorState::Error { path };
                                    return;
                                }
                            }
                        };

                        context.active_value = ValueWithSource {
                            inner: value,
                            source: Some(path),
                        };
                    }
                }
            }
        };

        self.state = EvaluatorState::Running {
            active_value: context.active_value.clone(),
        };

        self.advance();
    }

    fn next<'r>(&mut self, codebase: &'r Codebase) -> Next<'r> {
        let Some(context) = self.contexts.last() else {
            return Next::Finished {
                output: ValueWithSource {
                    inner: Value::Nothing,
                    source: None,
                },
            };
        };

        let Some(path) = context.nodes_from_root.last().copied() else {
            let output = context.active_value.clone();
            return Next::Finished { output };
        };

        if let EvaluatorState::Effect { effect, path } = self.state.clone() {
            return Next::Effect { effect, path };
        }

        match &codebase.node_at(&path).kind {
            NodeKind::Empty => Next::IgnoringSyntaxNode,
            NodeKind::Expression { expression } => {
                Next::Running { expression, path }
            }
            NodeKind::Recursion => Next::Recursing,
            NodeKind::Error { node: _ } => Next::Error { path },
        }
    }

    fn advance(&mut self) {
        if let Some(context) = self.contexts.last_mut() {
            context.nodes_from_root.pop();
        }
    }

    pub fn state(&self) -> &EvaluatorState {
        &self.state
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    /// # The nodes to be evaluated, sorted from root to leaf
    ///
    /// This is a subset of the full syntax tree. But it is not a tree itself,
    /// just a sequence of syntax node. If any of the nodes had multiple
    /// children (which would turn the sequence into a sub-tree), this would
    /// have caused a separate context to be created.
    pub nodes_from_root: Vec<NodePath>,

    pub active_value: ValueWithSource,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Next<'r> {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluatorState {
    Running { active_value: ValueWithSource },
    Effect { effect: Effect, path: NodePath },
    Finished { output: ValueWithSource },
    Error { path: NodePath },
}

impl EvaluatorState {
    #[cfg(test)]
    pub fn active_value(&self) -> Option<Value> {
        if let Self::Running { active_value, .. } = self {
            Some(active_value.inner.clone())
        } else {
            None
        }
    }

    pub fn path(&self) -> Option<&NodePath> {
        match self {
            Self::Running { active_value } => active_value.source.as_ref(),
            Self::Effect { path, .. } => Some(path),
            Self::Error { path } => Some(path),
            Self::Finished { .. } => None,
        }
    }
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
        runtime::{Evaluator, Value},
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
        evaluator.step(&codebase);

        assert_eq!(evaluator.state().active_value(), Some(Value::Nothing));
    }
}
