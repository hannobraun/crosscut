use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, Node, NodePath,
};

use super::{
    context::{Context, EvaluateUpdate},
    Effect, RuntimeState, Value, ValueWithSource,
};

#[derive(Debug)]
pub struct Evaluator {
    root: NodePath,
    contexts: Vec<Context>,
    state: RuntimeState,
}

impl Evaluator {
    pub fn new(root: NodePath, codebase: &Codebase) -> Self {
        let mut evaluator = Self {
            root,
            contexts: Vec::new(),
            state: RuntimeState::Running {
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

            if let Node::Expression {
                expression:
                    Expression::IntrinsicFunction {
                        intrinsic:
                            IntrinsicFunction::Literal {
                                literal: Literal::Function | Literal::Tuple,
                            },
                    },
                ..
            } = codebase.node_at(&path)
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
        self.state = RuntimeState::Running {
            active_value: active_value.clone(),
        };
        self.contexts.push(Context {
            nodes_from_root,
            active_value,
        });
    }

    pub fn provide_host_function_output(&mut self, value: Value) {
        let RuntimeState::Effect {
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
        self.state = RuntimeState::Running {
            active_value: context.active_value.clone(),
        };

        context.advance();
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

        self.state = RuntimeState::Effect { effect, path };
    }

    pub fn step(&mut self, codebase: &Codebase) {
        loop {
            match self.next(codebase) {
                Next::Running { expression, path } => {
                    match self.evaluate_expression(expression, path, codebase) {
                        EvaluateUpdate::UpdateState { new_state } => {
                            self.state = new_state;
                        }
                        EvaluateUpdate::NewContext { root, active_value } => {
                            self.evaluate(root, active_value, codebase);
                        }
                    }

                    break;
                }
                Next::IgnoringSyntaxNode => {
                    self.advance();
                    continue;
                }
                Next::ContextEvaluated => {
                    continue;
                }
                Next::Recursing => {
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
                    self.state = RuntimeState::Effect { effect, path };
                    return;
                }
                Next::Error { path } => {
                    self.state = RuntimeState::Error { path };
                    return;
                }
                Next::Finished { output } => {
                    self.state = RuntimeState::Finished { output };
                    return;
                }
            }
        }
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
            self.contexts.pop();

            if let Some(context) = self.contexts.last_mut() {
                match &mut context.active_value.inner {
                    Value::Tuple { elements } => {
                        elements.push(output.inner);
                    }
                    value => {
                        panic!(
                            "Expected value that would have created a context \
                            (got `{value:?}`)."
                        );
                    }
                }

                return Next::ContextEvaluated;
            } else {
                return Next::Finished { output };
            }
        };

        if let RuntimeState::Effect { effect, path } = self.state.clone() {
            return Next::Effect { effect, path };
        }

        match codebase.node_at(&path) {
            Node::Leaf => Next::IgnoringSyntaxNode,
            Node::Empty { .. } => Next::IgnoringSyntaxNode,
            Node::Expression { expression, .. } => {
                Next::Running { expression, path }
            }
            Node::Recursion { .. } => {
                let active_value = context.active_value.inner.clone();

                self.contexts.pop();
                self.evaluate(self.root, active_value, codebase);

                Next::Recursing
            }
            Node::Error { .. } => Next::Error { path },
        }
    }

    fn evaluate_expression(
        &mut self,
        expression: &Expression,
        path: NodePath,
        codebase: &Codebase,
    ) -> EvaluateUpdate {
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

        context.evaluate(expression, path, codebase)
    }

    fn advance(&mut self) {
        if let Some(context) = self.contexts.last_mut() {
            context.advance();
        }
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Next<'r> {
    Running {
        expression: &'r Expression,
        path: NodePath,
    },
    IgnoringSyntaxNode,
    ContextEvaluated,
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

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Codebase, Node},
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
            Node::Recursion {
                child: Some(*codebase.root().path.hash()),
            },
        );

        let mut evaluator = Evaluator::new(codebase.root().path, &codebase);
        evaluator.step(&codebase);

        assert_eq!(evaluator.state().active_value(), Some(Value::Nothing));
    }

    #[test]
    fn tail_call_elimination() {
        // The memory used by the evaluator should not grow, if a function is
        // tail-recursive.

        let mut codebase = Codebase::new();
        codebase.insert_as_parent_of(
            codebase.root().path,
            Node::Recursion {
                child: Some(*codebase.root().path.hash()),
            },
        );

        let mut evaluator = Evaluator::new(codebase.root().path, &codebase);
        assert_eq!(evaluator.contexts.len(), 1);

        evaluator.step(&codebase);
        assert_eq!(evaluator.contexts.len(), 1);
    }
}
