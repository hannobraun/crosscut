use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, NodeKind, NodePath, Type,
};

use super::{
    Effect, RuntimeState, Value,
    context::{Context, ContextNode},
};

#[derive(Debug)]
pub struct Evaluator {
    eval_stack: Vec<RuntimeNode>,
    call_stack: Vec<StackFrame>,
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
            },
        }
    }

    pub fn reset(&mut self, codebase: &Codebase) {
        *self = Self::new();
        self.call_function(codebase.root().path, Value::Nothing, codebase);
    }

    pub fn call_function(
        &mut self,
        root_path: NodePath,
        argument: Value,
        codebase: &Codebase,
    ) {
        let root_node = RuntimeNode::from_syntax_node(root_path, codebase);
        self.eval_stack.push(root_node);

        self.call_stack.push(StackFrame {
            root: root_path,
            argument: argument.clone(),
        });

        self.state = RuntimeState::Running { path: None };

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

        self.contexts.push(Context {
            next: previous,
            active_value: argument,
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
            path: Some(source.syntax_node),
        };

        context.advance();
        self.advance(value);
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
        if self.state.is_effect() {
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
                self.state = RuntimeState::Finished { output };
            }

            return;
        };

        let Some(mut node) = self.eval_stack.pop() else {
            // Evaluation stack is empty, which means there's nothing we can do.
            return;
        };

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
                // Seed all leaf nodes of a function with the function argument.
                // This is a weird thing to do, but it's how function arguments
                // work right now. We'll have real parameters in due time.
                if node.evaluated_children.inner.is_empty() {
                    if let Some(stack_frame) = self.call_stack.last() {
                        node.evaluated_children
                            .inner
                            .push(stack_frame.argument.clone());
                    }
                }

                break;
            };

            self.eval_stack.push(node);
            node = RuntimeNode::from_syntax_node(child, codebase);
        }

        let [kind_from_runtime_node, kind_from_context] =
            [node.syntax_node, next.syntax_node]
                .map(|path| codebase.node_at(path).node.kind());

        dbg!(kind_from_runtime_node);

        match kind_from_context {
            NodeKind::Empty { .. } => {
                context.advance();
                self.advance(node.evaluated_children.into_active_value());
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
                let path = next.syntax_node;
                match intrinsic {
                    IntrinsicFunction::Drop => {
                        context.active_value = Value::Nothing;
                        self.advance(Value::Nothing);
                    }
                    IntrinsicFunction::Eval => {
                        let body = match context.active_value {
                            Value::Function { body } => body,
                            ref active_value => {
                                self.unexpected_input(
                                    Type::Function,
                                    active_value.clone(),
                                    path,
                                );
                                self.contexts.push(context);
                                return;
                            }
                        };

                        self.contexts.push(context);
                        self.call_function(
                            NodePath { hash: body },
                            // Right now, the `eval` function doesn't support
                            // passing an argument to the function it evaluates.
                            Value::Nothing,
                            codebase,
                        );
                        return;
                    }
                    IntrinsicFunction::Identity => {
                        self.advance(
                            node.evaluated_children.into_active_value(),
                        );
                    }
                    IntrinsicFunction::Literal { literal } => {
                        let value = {
                            match *literal {
                                Literal::Function => {
                                    match &context.active_value {
                                        Value::Nothing => {}
                                        active_value => {
                                            self.unexpected_input(
                                                Type::Nothing,
                                                active_value.clone(),
                                                path,
                                            );
                                            self.eval_stack.push(node);
                                            self.contexts.push(context);
                                            return;
                                        }
                                    }

                                    let node = codebase.node_at(path);
                                    let mut children =
                                        node.children(codebase.nodes());

                                    let Some(child) = children.next() else {
                                        unreachable!(
                                            "Function literal must have a \
                                            child, or it wouldn't have been \
                                            resolved as a function literal."
                                        );
                                    };

                                    assert_eq!(
                                        children.count(),
                                        0,
                                        "Only nodes with one child can be \
                                        evaluated at this point.",
                                    );

                                    Value::Function {
                                        body: child.path.hash,
                                    }
                                }
                                Literal::Integer { value } => {
                                    match &context.active_value {
                                        Value::Nothing => {}
                                        active_value => {
                                            self.unexpected_input(
                                                Type::Nothing,
                                                active_value.clone(),
                                                path,
                                            );
                                            self.eval_stack.push(node);
                                            self.contexts.push(context);
                                            return;
                                        }
                                    }

                                    Value::Integer { value }
                                }
                                Literal::Tuple => {
                                    match &context.active_value {
                                        Value::Nothing => {}
                                        active_value => {
                                            self.unexpected_input(
                                                Type::Nothing,
                                                active_value.clone(),
                                                path,
                                            );
                                            self.eval_stack.push(node);
                                            self.contexts.push(context);
                                            return;
                                        }
                                    }

                                    assert!(
                                        node.children_to_evaluate.is_empty(),
                                        "Due to the loop above, which puts all \
                                        children of a node on the evaluation \
                                        stack, on top of that node, all \
                                        children of the tuple must be \
                                        evaluated by now.",
                                    );

                                    self.advance(Value::Tuple {
                                        elements: node.evaluated_children.inner,
                                    });

                                    let node2 = codebase.node_at(path);
                                    let mut children =
                                        node2.children(codebase.nodes());

                                    let Some(child) = children.next() else {
                                        unreachable!(
                                            "Tuple literal must have a child, \
                                            or it wouldn't have been resolved \
                                            as a tuple literal."
                                        );
                                    };

                                    assert_eq!(
                                        children.count(),
                                        0,
                                        "Only nodes with one child can be \
                                        evaluated at this point.",
                                    );

                                    context.active_value = Value::Tuple {
                                        elements: Vec::new(),
                                    };
                                    context.advance();

                                    self.contexts.push(context);
                                    self.call_function(
                                        child.path,
                                        Value::Nothing,
                                        codebase,
                                    );
                                    return;
                                }
                            }
                        };

                        context.active_value = value.clone();
                        self.advance(value);
                    }
                }

                context.advance();

                self.contexts.push(context);

                // We already restored the context. So we have to return now,
                // because the code below would do it again.
                return;
            }
            NodeKind::Recursion => {
                let path = self
                    .call_stack
                    .pop()
                    .map(|stack_frame| stack_frame.root)
                    .unwrap_or_else(|| codebase.root().path);

                let active_value = context.active_value.clone();
                self.call_function(path, active_value, codebase);

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

                // We don't want to advance the execution in any way when
                // encountering an error. So let's restore the node we pulled
                // from the evaluation stack earlier to where it was.
                self.eval_stack.push(node);
            }
        };

        // Restore the context that we took above. If that wasn't the right
        // thing to do, we'd have returned already.
        self.contexts.push(context);
    }

    fn unexpected_input(
        &mut self,
        expected: Type,
        actual: Value,
        path: NodePath,
    ) {
        self.state = RuntimeState::Effect {
            effect: Effect::UnexpectedInput { expected, actual },
            path,
        };
    }

    fn advance(&mut self, active_value: Value) {
        // When this is called, the current node has already been removed from
        // the stack.

        if let Some(parent) = self.eval_stack.last_mut() {
            self.state = RuntimeState::Running {
                path: Some(parent.syntax_node),
            };
            parent.evaluated_children.inner.push(active_value);
        } else {
            self.state = RuntimeState::Finished {
                output: active_value,
            };
        }
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }
}

#[derive(Debug)]
struct RuntimeNode {
    syntax_node: NodePath,
    children_to_evaluate: Vec<NodePath>,
    evaluated_children: EvaluatedChildren,
}

impl RuntimeNode {
    fn from_syntax_node(path: NodePath, codebase: &Codebase) -> Self {
        let root_node = codebase.node_at(path);

        Self {
            syntax_node: path,
            children_to_evaluate: root_node
                .children(codebase.nodes())
                .map(|located_node| located_node.path)
                .rev()
                .collect(),
            evaluated_children: EvaluatedChildren { inner: Vec::new() },
        }
    }
}

#[derive(Clone, Debug)]
struct EvaluatedChildren {
    inner: Vec<Value>,
}

impl EvaluatedChildren {
    pub fn into_active_value(mut self) -> Value {
        let value = self.inner.pop().unwrap_or(Value::Nothing);

        assert!(
            self.inner.is_empty(),
            "Expected a node to have zero or one children, but it has \
            multiple. This is a bug. Specifically, it is a mismatch of \
            expectations between compiler and evaluator.",
        );

        value
    }
}

#[derive(Debug)]
struct StackFrame {
    root: NodePath,
    argument: Value,
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Codebase, Node, NodeKind},
        runtime::{Evaluator, RuntimeState},
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
        assert!(evaluator.state().is_running());
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
