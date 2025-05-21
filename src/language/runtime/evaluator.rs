use crate::language::code::{Codebase, NodePath, Nodes, Type};

use super::{
    Effect, RuntimeState, Value,
    node::{RuntimeChild, RuntimeNode},
};

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
        self.apply_function(
            "_".to_string(),
            codebase.root().path,
            Value::nothing(),
            codebase.nodes(),
        );
    }

    pub fn apply_function(
        &mut self,
        parameter: String,
        body: NodePath,
        argument: Value,
        nodes: &Nodes,
    ) {
        self.eval_stack.push(RuntimeNode::new(body.clone(), nodes));
        self.call_stack.push(StackFrame {
            parameter,
            argument,
            root: body,
        });
    }

    pub fn exit_from_provided_function(&mut self, output: Value) {
        let RuntimeState::Effect {
            effect: Effect::ApplyProvidedFunction { .. },
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

        self.state = RuntimeState::Running;

        match node {
            RuntimeNode::Apply {
                expression: RuntimeChild::Unevaluated { ref path },
                ..
            }
            | RuntimeNode::Apply {
                expression: RuntimeChild::Evaluated { .. },
                argument: RuntimeChild::Unevaluated { ref path },
                ..
            } => {
                let path = path.clone();

                self.eval_stack.push(node);
                self.eval_stack
                    .push(RuntimeNode::new(path, codebase.nodes()));
            }
            RuntimeNode::Apply {
                expression:
                    RuntimeChild::Evaluated {
                        value: Value::Function { parameter, body },
                    },
                argument: RuntimeChild::Evaluated { value: argument },
                ..
            } => {
                self.eval_stack.push(RuntimeNode::PopStackFrame {
                    output: Value::nothing(),
                });
                self.apply_function(
                    parameter,
                    body,
                    argument,
                    codebase.nodes(),
                );
            }
            RuntimeNode::Apply {
                ref path,
                expression:
                    RuntimeChild::Evaluated {
                        value: Value::ProvidedFunction { ref name },
                    },
                argument:
                    RuntimeChild::Evaluated {
                        value: ref argument,
                    },
            } => {
                self.state = RuntimeState::Effect {
                    effect: Effect::ApplyProvidedFunction {
                        name: name.clone(),
                        input: argument.clone(),
                    },
                    path: path.clone(),
                };

                // A host function is not fully handled, until the handler has
                // provided its output. It might also trigger an effect, and
                // then we still need the node.
                self.eval_stack.push(node);
            }
            RuntimeNode::Apply {
                ref path,
                expression: RuntimeChild::Evaluated { ref value },
                ..
            } => {
                self.unexpected_input(
                    Type::Function,
                    value.clone(),
                    path.clone(),
                );
                self.eval_stack.push(node);
            }

            RuntimeNode::Expressions {
                ref mut to_evaluate,
                ..
            } if !to_evaluate.is_empty() => {
                let Some(child) = to_evaluate.pop() else {
                    // This could be prevented with an `if let` guard, but those
                    // are not stable yet, as of 2025-05-21:
                    // https://rust-lang.github.io/rfcs/2294-if-let-guard.html
                    unreachable!(
                        "The match guard above checks that there are values to \
                        evaluate."
                    );
                };

                self.eval_stack.push(node);
                self.eval_stack
                    .push(RuntimeNode::new(child, codebase.nodes()));
            }
            RuntimeNode::Expressions { mut evaluated, .. } => {
                let value = evaluated.pop().unwrap_or_else(Value::nothing);
                self.finish_evaluating_node(value);
            }

            RuntimeNode::Tuple {
                ref mut to_evaluate,
                ..
            } if !to_evaluate.is_empty() => {
                let Some(child) = to_evaluate.pop() else {
                    // This could be prevented with an `if let` guard, but those
                    // are not stable yet, as of 2025-05-16:
                    // https://rust-lang.github.io/rfcs/2294-if-let-guard.html
                    unreachable!(
                        "The match guard above checks that there are values to \
                        evaluate."
                    );
                };

                self.eval_stack.push(node);
                self.eval_stack
                    .push(RuntimeNode::new(child, codebase.nodes()));
            }
            RuntimeNode::Tuple { evaluated, .. } => {
                self.finish_evaluating_node(Value::Tuple { values: evaluated });
            }

            RuntimeNode::Empty => {
                self.finish_evaluating_node(Value::nothing());
            }
            RuntimeNode::Function { parameter, body } => {
                self.finish_evaluating_node(Value::Function {
                    parameter,
                    body,
                });
            }
            RuntimeNode::Identifier { name } => {
                let mut value = Value::ProvidedFunction { name: name.clone() };

                for stack_frame in self.call_stack.iter().rev() {
                    if stack_frame.parameter == name {
                        value = stack_frame.argument.clone();
                        break;
                    }
                }

                self.finish_evaluating_node(value);
            }
            RuntimeNode::Number { value } => {
                self.finish_evaluating_node(Value::Integer { value });
            }
            RuntimeNode::PopStackFrame { output } => {
                self.finish_evaluating_node(output);
                self.call_stack.pop();
            }
            RuntimeNode::Recursion => {
                let stack_frame =
                    self.call_stack.pop().unwrap_or_else(|| StackFrame {
                        parameter: "_".to_string(),
                        argument: Value::nothing(),
                        root: codebase.root().path,
                    });

                self.finish_evaluating_node(Value::Function {
                    parameter: stack_frame.parameter,
                    body: stack_frame.root,
                });
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
            parent.child_was_evaluated(output);
            RuntimeState::Running
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
struct StackFrame {
    parameter: String,
    argument: Value,
    root: NodePath,
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Apply, Codebase, Function, NodePath, SyntaxNode, Tuple},
        runtime::{Evaluator, RuntimeState, Value},
        tests::infra::ExpectChildren,
    };

    #[test]
    fn create_correct_path_for_function_value() {
        // When constructing a function value, the evaluator needs to create its
        // body's path. There are ways this could go wrong, so let's make sure
        // it doesn't.

        let mut codebase = Codebase::new();

        codebase.make_change(|change_set| {
            let node = Function::default().into_syntax_node(change_set.nodes);
            let hash = change_set.nodes.insert(node);

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(hash),
            );
        });

        let [expected_parameter, expected_body] =
            codebase.root().expect_children(codebase.nodes());

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);

        evaluator.step(&codebase);
        let RuntimeState::Finished {
            output: Value::Function { parameter: _, body },
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
            let apply = {
                let apply = Apply::default()
                    .with_expression(SyntaxNode::Recursion)
                    .with_argument(
                        Tuple::default().into_syntax_node(change_set.nodes),
                    )
                    .into_syntax_node(change_set.nodes);

                change_set.nodes.insert(apply)
            };

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(apply),
            )
        });

        let mut evaluator = Evaluator::new();
        evaluator.reset(&codebase);
        assert_eq!(evaluator.call_stack.len(), 1);

        // Not sure fo how many steps the code needs to run before it starts
        // over, but that's definitely more than enough.
        for _ in 0..1024 {
            evaluator.step(&codebase);
            assert!(matches!(evaluator.state(), RuntimeState::Running));
            assert!(evaluator.call_stack.len() <= 1);
        }
    }
}
