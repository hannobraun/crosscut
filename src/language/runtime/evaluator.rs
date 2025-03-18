use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, NodeKind, NodePath, Type,
};

use super::{Effect, RuntimeState, Value};

#[derive(Debug)]
pub struct Evaluator {
    eval_stack: Vec<RuntimeNode>,
    call_stack: Vec<StackFrame>,
    state: RuntimeState,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            eval_stack: Vec::new(),
            call_stack: Vec::new(),
            state: RuntimeState::Started,
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
        let root_node =
            RuntimeNode::from_syntax_node(root_path.clone(), codebase);
        self.eval_stack.push(root_node);

        self.call_stack.push(StackFrame {
            root: root_path,
            argument,
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

        // Now that its output has been provided, the host function is fully
        // handled. We can drop the node that triggered it.
        let Some(evaluated_node) = self.eval_stack.pop() else {
            unreachable!(
                "Effect has been triggered, but no node that could have \
                triggered it is available."
            );
        };

        self.finish_evaluating_node(evaluated_node.syntax_node, value);
    }

    pub fn trigger_effect(&mut self, effect: Effect) {
        let Some(node) = self.eval_stack.last() else {
            panic!(
                "Not allowed to trigger effect, if there is no active syntax \
                node that could trigger it."
            );
        };

        self.state = RuntimeState::Effect {
            effect,
            path: node.syntax_node.clone(),
        };
    }

    pub fn step(&mut self, codebase: &Codebase) {
        if self.state.is_effect() {
            return;
        }

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
            } = codebase.node_at(&node.syntax_node).node.kind()
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
                        node.evaluated_children.inner.push((
                            stack_frame.root.clone(),
                            stack_frame.argument.clone(),
                        ));
                    }
                }

                break;
            };

            self.eval_stack.push(node);
            node = RuntimeNode::from_syntax_node(child, codebase);
        }

        self.state = RuntimeState::Running {
            path: node.syntax_node.clone(),
        };

        match codebase.node_at(&node.syntax_node).node.kind() {
            NodeKind::Empty { .. } => {
                self.finish_evaluating_node(
                    node.syntax_node,
                    node.evaluated_children.into_active_value().1,
                );
            }
            NodeKind::Expression {
                expression: Expression::HostFunction { id },
                ..
            } => {
                self.state = RuntimeState::Effect {
                    effect: Effect::ApplyHostFunction {
                        id: *id,
                        input: node
                            .evaluated_children
                            .clone()
                            .into_active_value()
                            .1,
                    },
                    path: node.syntax_node.clone(),
                };

                // A host function is not fully handled, until the handler has
                // provided its output. It might also trigger an effect, and
                // then we still need the node.
                self.eval_stack.push(node);
            }
            NodeKind::Expression {
                expression: Expression::IntrinsicFunction { intrinsic },
                ..
            } => {
                match intrinsic {
                    IntrinsicFunction::Drop => {
                        self.finish_evaluating_node(
                            node.syntax_node,
                            Value::Nothing,
                        );
                    }
                    IntrinsicFunction::Eval => {
                        let (_, body) = match node
                            .evaluated_children
                            .clone()
                            .into_active_value()
                        {
                            (path, Value::Function { body }) => {
                                let Some(path) = path else {
                                    unreachable!(
                                        "`path` can only be `None`, if the \
                                        active value is `Nothing`, but here we \
                                        just verified that it is `Function`."
                                    );
                                };

                                (path, body)
                            }
                            (_, active_value) => {
                                self.unexpected_input(
                                    Type::Function,
                                    active_value,
                                    node.syntax_node,
                                );
                                return;
                            }
                        };

                        self.call_function(
                            NodePath { hash: body },
                            // Right now, the `eval` function doesn't support
                            // passing an argument to the function it evaluates.
                            Value::Nothing,
                            codebase,
                        );
                    }
                    IntrinsicFunction::Identity => {
                        self.finish_evaluating_node(
                            node.syntax_node,
                            node.evaluated_children.into_active_value().1,
                        );
                    }
                    IntrinsicFunction::Literal { literal } => {
                        let value = {
                            match *literal {
                                Literal::Function => {
                                    match node
                                        .evaluated_children
                                        .clone()
                                        .into_active_value()
                                        .1
                                    {
                                        Value::Nothing => {}
                                        active_value => {
                                            self.unexpected_input(
                                                Type::Nothing,
                                                active_value,
                                                node.syntax_node.clone(),
                                            );
                                            self.eval_stack.push(node);
                                            return;
                                        }
                                    }

                                    let node =
                                        codebase.node_at(&node.syntax_node);
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
                                    match node
                                        .evaluated_children
                                        .clone()
                                        .into_active_value()
                                        .1
                                    {
                                        Value::Nothing => {}
                                        active_value => {
                                            self.unexpected_input(
                                                Type::Nothing,
                                                active_value,
                                                node.syntax_node.clone(),
                                            );
                                            self.eval_stack.push(node);
                                            return;
                                        }
                                    }

                                    Value::Integer { value }
                                }
                                Literal::Tuple => {
                                    assert!(
                                        node.children_to_evaluate.is_empty(),
                                        "Due to the loop above, which puts all \
                                        children of a node on the evaluation \
                                        stack, on top of that node, all \
                                        children of the tuple must be \
                                        evaluated by now.",
                                    );

                                    Value::Tuple {
                                        elements: node
                                            .evaluated_children
                                            .inner
                                            .into_iter()
                                            .map(|(_, value)| value)
                                            .collect(),
                                    }
                                }
                            }
                        };

                        self.finish_evaluating_node(node.syntax_node, value);
                    }
                }
            }
            NodeKind::Recursion => {
                let path = self
                    .call_stack
                    .pop()
                    .map(|stack_frame| stack_frame.root)
                    .unwrap_or_else(|| codebase.root().path);

                let active_value =
                    node.evaluated_children.into_active_value().1;
                self.call_function(path, active_value, codebase);
            }
            NodeKind::Error { .. } => {
                self.state = RuntimeState::Error {
                    path: node.syntax_node.clone(),
                };

                // We don't want to advance the execution in any way when
                // encountering an error. So let's restore the node we pulled
                // from the evaluation stack earlier to where it was.
                self.eval_stack.push(node);
            }
        }
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

    fn finish_evaluating_node(
        &mut self,
        evaluated_node: NodePath,
        output: Value,
    ) {
        // When this is called, the current node has already been removed from
        // the stack.

        if let Some(parent) = self.eval_stack.last_mut() {
            self.state = RuntimeState::Running {
                path: parent.syntax_node.clone(),
            };
            parent
                .evaluated_children
                .inner
                .push((evaluated_node, output));
        } else {
            self.state = RuntimeState::Finished {
                output,
                path: evaluated_node,
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
        let root_node = codebase.node_at(&path);

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
    inner: Vec<(NodePath, Value)>,
}

impl EvaluatedChildren {
    pub fn into_active_value(mut self) -> (Option<NodePath>, Value) {
        let (path, value) = self
            .inner
            .pop()
            .map(|(path, value)| (Some(path), value))
            .unwrap_or((None, Value::Nothing));

        assert!(
            self.inner.is_empty(),
            "Expected a node to have zero or one children, but it has \
            multiple. This is a bug. Specifically, it is a mismatch of \
            expectations between compiler and evaluator.",
        );

        (path, value)
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
            let node = Node::new(NodeKind::Recursion, []);
            change_set.replace(&root, node)
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
            change_set.replace(&root, Node::new(NodeKind::Recursion, []))
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);
        assert_eq!(evaluator.call_stack.len(), 1);

        evaluator.step(&codebase);
        assert!(matches!(evaluator.state(), RuntimeState::Running { .. }));
        assert_eq!(evaluator.call_stack.len(), 1);
    }
}
