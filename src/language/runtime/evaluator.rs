use crate::language::code::{Codebase, Node, NodePath, SiblingIndex, Type};

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
        self.apply_function_raw(codebase.root().path, Value::Nothing, codebase);
    }

    /// # Apply a function using the current node as source
    ///
    /// Calling this function is appropriate, if the evaluation originates from
    /// the current syntax node. That would typically mean, that the current
    /// syntax node is an application of a provided function, and this
    /// evaluation is part of handling that function application.
    ///
    /// If this isn't the case, please call [`Evaluator::eval_function_raw`]
    /// instead.
    pub fn apply_function_from_current_node(
        &mut self,
        body: NodePath,
        argument: Value,
        codebase: &Codebase,
    ) {
        let Some(node) = self.eval_stack.pop() else {
            panic!(
                "Trying to apply a function from a node, but no node is \
                available."
            );
        };

        self.apply_function_raw(body, argument, codebase);

        self.state = RuntimeState::Running {
            path: node.syntax_node,
        };
    }

    /// # Apply a function without considering where that might originate
    ///
    /// This function just does the bare minimum of starting the evaluation. It
    /// doesn't consider, if the evaluation might have originated from a syntax
    /// node, nor does it do anything about the current state.
    ///
    /// Calling this function is appropriate, if the evaluation originates from
    /// outside of the source code. The caller is expected to take care of
    /// anything else that might happen to make this work correctly.
    ///
    /// If there is a current node that the evaluation originates from a syntax
    /// node, please call [`Evaluator::eval_function_from_current_node`]
    /// instead.
    pub fn apply_function_raw(
        &mut self,
        body: NodePath,
        argument: Value,
        codebase: &Codebase,
    ) {
        self.eval_stack
            .push(RuntimeNode::from_syntax_node(body.clone(), codebase));

        self.call_stack.push(StackFrame {
            root: body,
            argument,
        });
    }

    pub fn exit_from_provided_function(&mut self, output: Value) {
        let RuntimeState::Effect {
            effect: Effect::ProvidedFunction { .. },
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
        let Some(_) = self.eval_stack.pop() else {
            unreachable!(
                "Effect has been triggered, but no node that could have \
                triggered it is available."
            );
        };

        self.finish_evaluating_node(output);
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
            if let Node::LiteralFunction { .. } | Node::Error { .. } =
                codebase.node_at(&node.syntax_node).node
            {
                // We encountered a function literal and an error node. Either
                // means that we need to stop here.
                //
                // If this is a function literal, then not stopping here would
                // cause the function to be evaluated right away, where it is
                // defined, defeating the purpose of defining a function
                // literal.
                //
                // With an error, things are a bit less clear-cut. Most of the
                // time, it would make sense to evaluate any valid code up to an
                // error. But if the error was supposed to be a function
                // literal, then just evaluating its body would be wild and
                // unexpected.
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

        self.state = RuntimeState::Running {
            path: node.syntax_node.clone(),
        };

        match codebase.node_at(&node.syntax_node).node {
            Node::Empty { .. } => {
                self.finish_evaluating_node(
                    node.evaluated_children.into_active_value(),
                );
            }
            Node::ProvidedFunction { id, .. } => {
                self.state = RuntimeState::Effect {
                    effect: Effect::ProvidedFunction {
                        id: *id,
                        input: node
                            .evaluated_children
                            .clone()
                            .into_active_value(),
                    },
                    path: node.syntax_node.clone(),
                };

                // A host function is not fully handled, until the handler has
                // provided its output. It might also trigger an effect, and
                // then we still need the node.
                self.eval_stack.push(node);
            }
            Node::LiteralFunction { parameter: _, body } => {
                match node.evaluated_children.clone().into_active_value() {
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

                let body = NodePath::new(
                    *body,
                    Some(node.syntax_node),
                    SiblingIndex { index: 1 },
                    codebase.nodes(),
                );

                self.finish_evaluating_node(Value::Function { body });
            }
            Node::LiteralNumber { value } => {
                match node.evaluated_children.clone().into_active_value() {
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

                self.finish_evaluating_node(Value::Integer { value: *value });
            }
            Node::LiteralTuple { .. } => {
                assert!(
                    node.children_to_evaluate.is_empty(),
                    "Due to the loop above, which puts all children of a node \
                    on the evaluation stack, on top of that node, all children \
                    of the tuple must be evaluated by now.",
                );

                self.finish_evaluating_node(Value::Tuple {
                    elements: node
                        .evaluated_children
                        .inner
                        .into_iter()
                        .collect(),
                });
            }
            Node::Recursion { .. } => {
                let path = self
                    .call_stack
                    .pop()
                    .map(|stack_frame| stack_frame.root)
                    .unwrap_or_else(|| codebase.root().path);

                let active_value = node.evaluated_children.into_active_value();
                self.apply_function_raw(path, active_value, codebase);
            }
            Node::Error { .. } => {
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

    fn finish_evaluating_node(&mut self, output: Value) {
        // When this is called, the current node has already been removed from
        // the stack.

        let new_state = if let Some(parent) = self.eval_stack.last_mut() {
            parent.evaluated_children.inner.push(output);

            RuntimeState::Running {
                path: parent.syntax_node.clone(),
            }
        } else {
            RuntimeState::Finished { output }
        };

        self.state = new_state;
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
    use itertools::Itertools;

    use crate::language::{
        code::{Codebase, Node, NodePath},
        runtime::{Evaluator, RuntimeState, Value},
    };

    #[test]
    fn create_correct_path_for_function_value() {
        // When constructing a function value, the evaluator needs to create its
        // body's path. There are ways this could go wrong, so let's make sure
        // it doesn't.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            let parameter = change_set.add(Node::LiteralNumber { value: 0 });
            let body = change_set.add(Node::Empty { child: None });

            let function =
                change_set.add(Node::LiteralFunction { parameter, body });

            change_set.replace(&root, &NodePath::for_root(function));
        });

        let [expected_parameter, expected_body] = codebase
            .root()
            .children(codebase.nodes())
            .collect_array()
            .unwrap();

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);

        evaluator.step(&codebase);
        let RuntimeState::Finished {
            output: Value::Function { body },
        } = evaluator.state()
        else {
            panic!();
        };

        // At some point the parameter will be part of the function value. Then
        // we'll need to check against this expected value.
        let _ = expected_parameter;
        assert_eq!(body, &expected_body.path);
    }

    #[test]
    fn handle_bare_recursion() {
        // Recursion can quite naturally be implemented in a way that results in
        // an endless loop within `step`, if the evaluated expression consists
        // of nothing but a `self`. And in fact, that's what the first draft
        // did.

        let mut codebase = Codebase::new();

        let root = codebase.root().path;
        codebase.make_change(|change_set| {
            let hash = change_set.add(Node::Recursion { argument: None });
            change_set.replace(&root, &NodePath::for_root(hash))
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
            let hash = change_set.add(Node::Recursion { argument: None });
            change_set.replace(&root, &NodePath::for_root(hash))
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);
        assert_eq!(evaluator.call_stack.len(), 1);

        evaluator.step(&codebase);
        assert!(matches!(evaluator.state(), RuntimeState::Running { .. }));
        assert_eq!(evaluator.call_stack.len(), 1);
    }
}
