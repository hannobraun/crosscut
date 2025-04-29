use itertools::Itertools;

use crate::language::{
    code::{Codebase, Expression, Function, NodePath, SiblingIndex, Type},
    packages::FunctionId,
};

use super::{Effect, RuntimeState, Value};

#[derive(Debug)]
pub struct Evaluator {
    eval_stack: Vec<RuntimeExpression>,
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
        self.apply_function_raw(codebase.root().path, codebase);
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
    pub fn apply_function_raw(&mut self, body: NodePath, codebase: &Codebase) {
        self.eval_stack
            .push(RuntimeExpression::new(body.clone(), codebase));

        self.call_stack.push(StackFrame { root: body });
    }

    pub fn exit_from_provided_function(&mut self, output: Value) {
        let RuntimeState::Effect {
            effect: Effect::ProvidedFunction { .. },
            ..
        } = &self.state
        else {
            panic!(
                "Trying to provide host function output, but no host function \
                is currently being applied.\n\
                \n\
                Current state is `{:#?}`.",
                self.state,
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
        let Some(path) = self.state.path() else {
            panic!(
                "Tried to trigger effect, but the runtime is not in a state \
                that could lead to that.\n\
                \n\
                Expected a state that points to an expression that could be \
                the source of the effect. Instead, got this:\n\
                {:#?}",
                self.state
            );
        };

        self.state = RuntimeState::Effect {
            effect,
            path: path.clone(),
        };
    }

    pub fn step(&mut self, codebase: &Codebase) {
        if self.state.is_effect() {
            return;
        }

        let Some(mut node) = self.eval_stack.pop() else {
            // Evaluation stack is empty, which means there's nothing we can do.

            if !self.state.is_finished() {
                self.state = RuntimeState::Finished {
                    output: Value::nothing(),
                };
            }

            return;
        };

        // For the most part, we need to evaluate a node's children before we
        // can evaluate the node itself. This loop makes sure that `node` is a
        // node that can be evaluated, and that all its parents are on the
        // evaluation stack, so they can be evaluated later.
        loop {
            if let Expression::Function { .. } | Expression::Error { .. } =
                codebase.node_at(&node.path).node
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
                break;
            };

            self.eval_stack.push(node);
            node = RuntimeExpression::new(child, codebase);
        }

        self.state = RuntimeState::Running {
            path: node.path.clone(),
        };

        match node.kind {
            RuntimeExpressionKind::Apply => {
                let Some([function, argument]) = node
                    .clone()
                    .evaluated_children
                    .inner
                    .into_iter()
                    .collect_array()
                else {
                    unreachable!(
                        "`Node::Application must have two children. If it \
                        doesn't, that is a bug. Specifically, it is a mismatch \
                        between the compiler and the evaluator."
                    );
                };

                match function {
                    Value::Function { body } => {
                        self.apply_function_raw(body, codebase);
                    }
                    Value::ProvidedFunction { id } => {
                        self.state = RuntimeState::Effect {
                            effect: Effect::ProvidedFunction {
                                id,
                                input: argument,
                            },
                            path: node.path.clone(),
                        };

                        // A host function is not fully handled, until the
                        // handler has provided its output. It might also
                        // trigger an effect, and then we still need the node.
                        self.eval_stack.push(node);
                    }
                    value => {
                        self.unexpected_input(
                            Type::Function,
                            value.clone(),
                            node.path.clone(),
                        );
                        self.eval_stack.push(node);
                    }
                }
            }
            RuntimeExpressionKind::Empty => {
                self.finish_evaluating_node(
                    node.evaluated_children.into_active_value(),
                );
            }
            RuntimeExpressionKind::Function {
                function: Function { parameter: _, body },
            } => {
                let body = NodePath::new(
                    body,
                    Some(node.path),
                    SiblingIndex { index: 1 },
                    codebase.nodes(),
                );

                self.finish_evaluating_node(Value::Function { body });
            }
            RuntimeExpressionKind::Number { value } => {
                self.finish_evaluating_node(Value::Integer { value });
            }
            RuntimeExpressionKind::ProvidedFunction { id, .. } => {
                self.finish_evaluating_node(Value::ProvidedFunction { id });
            }
            RuntimeExpressionKind::Recursion => {
                let body = self
                    .call_stack
                    .pop()
                    .map(|stack_frame| stack_frame.root)
                    .unwrap_or_else(|| codebase.root().path);

                self.finish_evaluating_node(Value::Function { body });
            }
            RuntimeExpressionKind::Tuple => {
                assert!(
                    node.children_to_evaluate.is_empty(),
                    "Due to the loop above, which puts all children of a node \
                    on the evaluation stack, on top of that node, all children \
                    of the tuple must be evaluated by now.",
                );

                self.finish_evaluating_node(Value::Tuple {
                    values: node.evaluated_children.inner.into_iter().collect(),
                });
            }
            RuntimeExpressionKind::Error => {
                self.state = RuntimeState::Error {
                    path: node.path.clone(),
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
                path: parent.path.clone(),
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

#[derive(Clone, Debug)]
struct RuntimeExpression {
    path: NodePath,
    kind: RuntimeExpressionKind,
    children_to_evaluate: Vec<NodePath>,
    evaluated_children: EvaluatedChildren,
}

impl RuntimeExpression {
    fn new(path: NodePath, codebase: &Codebase) -> Self {
        let expression = codebase.node_at(&path);

        let kind = match expression.node.clone() {
            Expression::Apply { .. } => RuntimeExpressionKind::Apply,
            Expression::Empty => RuntimeExpressionKind::Empty,
            Expression::Function { function } => {
                RuntimeExpressionKind::Function { function }
            }
            Expression::Number { value } => {
                RuntimeExpressionKind::Number { value }
            }
            Expression::ProvidedFunction { id } => {
                RuntimeExpressionKind::ProvidedFunction { id }
            }
            Expression::Recursion => RuntimeExpressionKind::Recursion,
            Expression::Tuple { .. } => RuntimeExpressionKind::Tuple,
            Expression::Error { .. } => RuntimeExpressionKind::Error,
        };

        Self {
            path,
            kind,
            children_to_evaluate: expression
                .children(codebase.nodes())
                .map(|located_node| located_node.path)
                .rev()
                .collect(),
            evaluated_children: EvaluatedChildren { inner: Vec::new() },
        }
    }
}

#[derive(Clone, Debug)]
enum RuntimeExpressionKind {
    Apply,
    Empty,
    Function { function: Function },
    Number { value: i32 },
    ProvidedFunction { id: FunctionId },
    Recursion,
    Tuple,
    Error,
}

#[derive(Clone, Debug)]
struct EvaluatedChildren {
    inner: Vec<Value>,
}

impl EvaluatedChildren {
    pub fn into_active_value(mut self) -> Value {
        let value = self.inner.pop().unwrap_or(Value::nothing());

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
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Children, Codebase, Expression, Function, NodePath},
        runtime::{Evaluator, RuntimeState, Value},
        tests::infra::LocatedNodeExt,
    };

    #[test]
    fn create_correct_path_for_function_value() {
        // When constructing a function value, the evaluator needs to create its
        // body's path. There are ways this could go wrong, so let's make sure
        // it doesn't.

        let mut codebase = Codebase::new();

        codebase.make_change(|change_set| {
            let parameter =
                change_set.nodes.insert(Expression::Number { value: 0 });
            let body = change_set.nodes.insert(Expression::Empty);

            let function = change_set.nodes.insert(Expression::Function {
                function: Function { parameter, body },
            });

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(function),
            );
        });

        let [expected_parameter, expected_body] =
            codebase.root().expect_children(codebase.nodes());

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
    fn tail_call_elimination() {
        // The memory used by the evaluator should not grow, if a function is
        // tail-recursive.

        let mut codebase = Codebase::new();

        codebase.make_change(|change_set| {
            let recursion = change_set.nodes.insert(Expression::Recursion);
            let argument = change_set.nodes.insert(Expression::Tuple {
                values: Children::new([]),
            });

            let apply = change_set.nodes.insert(Expression::Apply {
                function: recursion,
                argument,
            });

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(apply),
            )
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);
        assert_eq!(evaluator.call_stack.len(), 1);

        evaluator.step(&codebase); // recursion
        evaluator.step(&codebase); // argument
        evaluator.step(&codebase); // apply
        assert!(matches!(evaluator.state(), RuntimeState::Running { .. }));
        assert_eq!(evaluator.call_stack.len(), 1);
    }
}
