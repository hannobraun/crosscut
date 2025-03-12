use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, NodeKind, NodePath,
};

use super::{
    Effect, RuntimeState, Value,
    context::{Context, ContextNode, EvaluateUpdate},
};

#[derive(Debug)]
pub struct Evaluator {
    eval_stack: Vec<RuntimeNode>,
    call_stack: Vec<NodePath>,
    contexts: Vec<Context>,
    state: RuntimeState,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            eval_stack: Vec::new(),
            call_stack: Vec::new(),
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
        root_path: NodePath,
        active_value: Value,
        codebase: &Codebase,
    ) {
        self.eval_stack.push(RuntimeNode::from_syntax_node(
            root_path,
            active_value.clone(),
            codebase,
        ));

        let mut path = root_path;
        let mut previous = None;

        loop {
            previous = Some(ContextNode {
                syntax_node: path,
                parent: previous.map(Box::new),
            });

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
            | NodeKind::Error { .. } = codebase.node_at(path).node.kind()
            {
                // We have already pushed the function literal, which means
                // we're going to evaluate it. But we need to stop here, since
                // we don't want to evaluate the function itself right now.
                break;
            }

            let node = codebase.node_at(path);
            let mut children = node.children(codebase.nodes());

            if let Some(child) = children.next() {
                assert_eq!(
                    children.count(),
                    0,
                    "Only nodes with one child can be evaluated at this point.",
                );

                path = child.path;
                continue;
            } else {
                break;
            }
        }

        self.state = RuntimeState::Running {
            active_value: active_value.clone(),
            path: None,
        };
        self.call_stack.push(root_path);
        self.contexts.push(Context {
            next: previous,
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
                "Host function is being applied, but there is no active \
                context. This should not be possible, because without a \
                context, what could have triggered the effect?"
            );
        };
        let Some(source) = &context.next else {
            unreachable!(
                "Host function is being applied, but there doesn't seem to be \
                a syntax node that could have triggered it."
            );
        };

        context.active_value = value.clone();
        self.state = RuntimeState::Running {
            active_value: value,
            path: Some(source.syntax_node),
        };

        context.advance();
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        let Some(context) = self.contexts.last_mut() else {
            panic!(
                "There is no active context. Not allowed to trigger effect."
            );
        };
        let Some(source) = &context.next else {
            panic!(
                "Not allowed to trigger effect, if there is no active syntax \
                node that could trigger it."
            );
        };

        self.state = RuntimeState::Effect {
            effect,
            path: source.syntax_node,
        };
    }

    pub fn step(&mut self, codebase: &Codebase) {
        if let RuntimeState::Effect { .. } = &self.state {
            return;
        }

        let Some(mut node) = self.eval_stack.pop() else {
            // Evaluation stack is empty, which means we're not running.

            self.state = RuntimeState::Finished {
                output: Value::Nothing,
                path: None,
            };

            return;
        };

        dbg!(&node.syntax_node);
        dbg!(&node.active_value);
        dbg!(&node.children_to_evaluate);
        dbg!(&node.evaluated_children);

        // For the most part, we need to evaluate a node's children before we
        // can evaluate the node itself. This loop makes sure that `node` is a
        // node that can be evaluated, and that all its parents are on the
        // evaluation stack, so they can be evaluated later.
        loop {
            if let NodeKind::Expression {
                expression:
                    Expression::IntrinsicFunction {
                        intrinsic:
                            IntrinsicFunction::Literal {
                                literal: Literal::Function,
                            },
                    },
            } = codebase.node_at(node.syntax_node).node.kind()
            {
                // If this were any other node, we'd need to evaluate its
                // children first. But function nodes are different. Their child
                // should only be evaluated, when the function is applied.
                break;
            }

            let Some(child) = node.children_to_evaluate.pop() else {
                break;
            };

            self.eval_stack.push(node);
            node =
                RuntimeNode::from_syntax_node(child, Value::Nothing, codebase);
        }

        self.eval_stack.push(node);

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

        let Some(next) = &context.next else {
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

        match codebase.node_at(next.syntax_node).node.kind() {
            NodeKind::Empty { .. } => {
                context.advance();
            }
            NodeKind::Expression {
                expression: Expression::HostFunction { id },
                ..
            } => {
                self.state = RuntimeState::Effect {
                    effect: Effect::ApplyHostFunction {
                        id: *id,
                        input: context.active_value.clone(),
                    },
                    path: next.syntax_node,
                };
            }
            NodeKind::Expression {
                expression: Expression::IntrinsicFunction { intrinsic },
                ..
            } => {
                let update = context.evaluate_intrinsic_function(
                    intrinsic,
                    next.syntax_node,
                    codebase,
                );
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
            NodeKind::Recursion => {
                let path = self
                    .call_stack
                    .pop()
                    .unwrap_or_else(|| codebase.root().path);

                let active_value = context.active_value.clone();
                self.push_context(path, active_value, codebase);

                // Return here, to bypass restoring the context. We already
                // created a new one with the call above, and the old one has
                // become redundant.
                //
                // This is tail call elimination.
                return;
            }
            NodeKind::Error { .. } => {
                self.state = RuntimeState::Error {
                    path: next.syntax_node,
                };
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

#[derive(Debug)]
pub struct RuntimeNode {
    pub syntax_node: NodePath,
    pub active_value: Value,
    pub children_to_evaluate: Vec<NodePath>,
    pub evaluated_children: Vec<Value>,
}

impl RuntimeNode {
    fn from_syntax_node(
        path: NodePath,
        active_value: Value,
        codebase: &Codebase,
    ) -> Self {
        let root_node = codebase.node_at(path);

        Self {
            syntax_node: path,
            active_value,
            children_to_evaluate: root_node
                .children(codebase.nodes())
                .map(|located_node| located_node.path)
                .rev()
                .collect(),
            evaluated_children: Vec::new(),
        }
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
        assert_eq!(evaluator.call_stack.len(), 1);
        assert_eq!(evaluator.contexts.len(), 1);

        evaluator.step(&codebase);
        assert!(matches!(evaluator.state(), RuntimeState::Running { .. }));
        assert_eq!(evaluator.call_stack.len(), 1);
        assert_eq!(evaluator.contexts.len(), 1);
    }
}
