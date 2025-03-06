use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, NodeKind, NodePath,
};

use super::{
    Effect, RuntimeState, Value,
    context::{Context, EvaluateUpdate},
};

#[derive(Debug)]
pub struct Evaluator {
    contexts: Vec<Context>,
    state: RuntimeState,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
            state: RuntimeState::Finished {
                output: Value::Nothing,
                path: None,
            },
        }
    }

    pub fn reset(&mut self, codebase: &Codebase) {
        *self = Self::new();
        self.push_context(codebase.root().path, Value::Nothing, codebase);
    }

    pub fn push_context(
        &mut self,
        root: NodePath,
        active_value: Value,
        codebase: &Codebase,
    ) {
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
                ..
            }
            | NodeKind::Error { .. } = codebase.node_at(&path).kind()
            {
                // We have already pushed the function literal, which means
                // we're going to evaluate it. But we need to stop here, since
                // we don't want to evaluate the function itself right now.
                break;
            }

            let mut children = codebase.children_of(&path).to_paths();

            if let Some(child) = children.next() {
                assert_eq!(
                    children.count(),
                    0,
                    "Only nodes with one child can be evaluated at this point.",
                );

                path = child;
                continue;
            } else {
                break;
            }
        }

        self.state = RuntimeState::Running {
            active_value: active_value.clone(),
            path: None,
        };
        self.contexts.push(Context {
            root,
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

        context.active_value = value.clone();
        self.state = RuntimeState::Running {
            active_value: value,
            path: Some(source),
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
        if let RuntimeState::Effect { .. } = &self.state {
            return;
        }

        // Take the current context. Depending on how things will go, we'll
        // restore it below; or do nothing, if it turns out we actually need to
        // remove it.
        //
        // Doing it this way makes some of the code below simpler or more
        // efficient, for lifetime or cloning reasons.
        let Some(mut context) = self.contexts.pop() else {
            // No context is available, which means we're not running.

            self.state = RuntimeState::Finished {
                output: Value::Nothing,
                path: None,
            };

            return;
        };

        let Some(path) = context.nodes_from_root.last().copied() else {
            // The context has no syntax tree remaining, which means we're done
            // with it.

            let output = context.active_value;

            if let Some(context) = self.contexts.last_mut() {
                match &mut context.active_value {
                    Value::Function { .. } => {
                        // If the context was created from a function, that
                        // means something has evaluated it.
                        context.active_value = output;
                        context.advance();
                    }
                    Value::Tuple { elements } => {
                        elements.push(output);
                    }
                    value => {
                        panic!(
                            "Expected value that would have created a context, \
                            got `{value}`."
                        );
                    }
                }
            } else {
                self.state = RuntimeState::Finished {
                    output,
                    path: self.state.path().cloned(),
                };
            }

            return;
        };

        match codebase.node_at(&path).kind() {
            NodeKind::Empty { .. } => {
                context.advance();
            }
            NodeKind::Expression {
                expression: Expression::HostFunction { id },
                ..
            } => {
                let effect = context.evaluate_host_function(*id);
                self.state = RuntimeState::Effect { effect, path };
            }
            NodeKind::Expression {
                expression: Expression::IntrinsicFunction { intrinsic },
                ..
            } => {
                let update = context
                    .evaluate_intrinsic_function(intrinsic, path, codebase);
                self.contexts.push(context);

                // The context is now restored. This means we can apply the
                // update from the evaluation now.

                match update {
                    EvaluateUpdate::UpdateState { new_state } => {
                        self.state = new_state;
                    }
                    EvaluateUpdate::PushContext { root, active_value } => {
                        self.push_context(root, active_value, codebase);
                    }
                }

                // We already restored the context. So we have to return now,
                // because the code below would do it again.
                return;
            }
            NodeKind::Recursion { .. } => {
                let active_value = context.active_value.clone();
                self.push_context(context.root, active_value, codebase);

                // Return here, to bypass restoring the context. We already
                // created a new one with the call above, and the old one has
                // become redundant.
                //
                // This is tail call elimination.
                return;
            }
            NodeKind::Error { .. } => {
                self.state = RuntimeState::Error { path };
            }
        };

        // Restore the context that we took above. If that wasn't the right
        // thing to do, we'd have returned already.
        self.contexts.push(context);
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Codebase, Node, NodeKind},
        runtime::{Evaluator, RuntimeState, Value},
    };

    #[test]
    fn handle_bare_recursion() {
        // Recursion can quite naturally be implemented in a way that results in
        // an endless loop within `step`, if the evaluated expression consists
        // of nothing but a `self`. And in fact, that's what the first draft
        // did.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            change_set.replace(root, Node::new(NodeKind::Recursion, []))
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);

        evaluator.step(&codebase);
        assert_eq!(evaluator.state().active_value(), Some(&Value::Nothing));
    }

    #[test]
    fn tail_call_elimination() {
        // The memory used by the evaluator should not grow, if a function is
        // tail-recursive.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            change_set.replace(root, Node::new(NodeKind::Recursion, []))
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);
        assert_eq!(evaluator.contexts.len(), 1);

        evaluator.step(&codebase);
        assert!(matches!(evaluator.state(), RuntimeState::Running { .. }));
        assert_eq!(evaluator.contexts.len(), 1);
    }
}
