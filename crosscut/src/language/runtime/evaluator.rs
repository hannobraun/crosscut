use std::collections::VecDeque;

use itertools::Itertools;

use crate::language::code::{Codebase, NodePath, Nodes, Type};

use super::{
    Effect, RuntimeState, Value,
    eval_step::{DerivedEvalStep, EvalStep, QueuedEvalStep, SyntheticEvalStep},
};

#[derive(Debug, Default)]
pub struct Evaluator {
    /// # Stack of evaluation steps that are currently in progress
    ///
    /// An evaluation step is in progress, if it has just been added and has not
    /// started evaluating yet, or if it has children that are currently being
    /// evaluated. In the latter case, those children would also be on the
    /// stack, higher than their parent.
    eval_stack: Vec<EvalStep>,

    /// # Evaluation steps that will be evaluated in the future
    ///
    /// These are the children of evaluation steps that are currently on the
    /// evaluation stack. The children of a step are all added to the queue when
    /// the step itself is added to the stack.
    ///
    /// New steps are added to the front of the queue, meaning that the first
    /// child of a step and all its descendants are evaluated before the second
    /// child of the step is taken off the queue.
    eval_queue: VecDeque<QueuedEvalStep>,

    evaluated_children: Vec<Value>,

    call_stack: Vec<StackFrame>,
    state: RuntimeState,
}

impl Evaluator {
    pub fn update(&mut self, codebase: &Codebase) {
        for step in &mut self.eval_stack {
            match step {
                EvalStep::Derived { .. } => {}
                EvalStep::Synthetic { .. } => {}
            }
        }

        self.reset(codebase);
    }

    pub fn reset(&mut self, codebase: &Codebase) {
        *self = Self::default();
        self.apply_function(
            "".to_string(),
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
        self.eval_stack.push(EvalStep::derived(
            body.clone(),
            &mut self.eval_queue,
            nodes,
        ));
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

        self.finish_step(output);
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

        let Some(mut eval_step) = self.eval_stack.pop() else {
            // Evaluation stack is empty, which means there's nothing we can do.

            if !self.state.is_finished() {
                self.state = RuntimeState::Finished {
                    output: Value::nothing(),
                };
            }

            return;
        };

        self.state = RuntimeState::Running;

        let mut evaluated_children = if let EvalStep::Derived {
            path,
            num_children,
            children_to_evaluate,
            ..
        } = &mut eval_step
        {
            if *children_to_evaluate > 0 {
                let Some(child) = self.eval_queue.pop_front() else {
                    unreachable!(
                        "The match guard above checks that there are values to \
                        evaluate."
                    );
                };
                assert_eq!(&child.parent, path);

                *children_to_evaluate -= 1;

                self.eval_stack.push(eval_step);
                self.eval_stack.push(EvalStep::derived(
                    child.path,
                    &mut self.eval_queue,
                    codebase.nodes(),
                ));

                // We have to evaluate the child first, and we'll start with
                // that on the next step. No need to look more closely at the
                // current step right now.
                return;
            } else {
                let Some(index) =
                    self.evaluated_children.len().checked_sub(*num_children)
                else {
                    unreachable!(
                        "All children have been evaluated, so there must be at \
                        least `num_children` evaluated children available."
                    );
                };

                let children =
                    self.evaluated_children.drain(index..).collect::<Vec<_>>();
                assert_eq!(children.len(), *num_children);

                children
            }
        } else {
            Vec::new()
        };

        match eval_step {
            EvalStep::Derived {
                step: DerivedEvalStep::Apply { is_tail_call },
                ref path,
                ..
            } => {
                let Some([function, argument]) =
                    evaluated_children.iter().collect_array()
                else {
                    unreachable!(
                        "Apply nodes have exactly two children. And unless \
                        both have been evaluated, the child handling code \
                        above wouldn't have let us arrive here."
                    );
                };

                match function {
                    Value::Function { parameter, body } => {
                        if is_tail_call {
                            self.call_stack.pop();
                        } else {
                            self.eval_stack.push(EvalStep::Synthetic {
                                step: SyntheticEvalStep::PopStackFrame,
                            });
                        }

                        self.apply_function(
                            parameter.clone(),
                            body.clone(),
                            argument.clone(),
                            codebase.nodes(),
                        );
                    }
                    Value::ProvidedFunction { name } => {
                        self.state = RuntimeState::Effect {
                            effect: Effect::ApplyProvidedFunction {
                                name: name.clone(),
                                input: argument.clone(),
                            },
                            path: path.clone(),
                        };

                        // A provided function is not fully handled, until the
                        // handler has provided its output. It might also
                        // trigger an effect, and then we still need the node.
                        self.eval_stack.push(eval_step);
                    }
                    value => {
                        self.unexpected_input(
                            Type::Function,
                            value.clone(),
                            path.clone(),
                        );
                        self.eval_stack.push(eval_step);
                    }
                }
            }

            EvalStep::Derived {
                step: DerivedEvalStep::Body,
                ..
            } => {
                let value =
                    evaluated_children.pop().unwrap_or_else(Value::nothing);
                self.finish_step(value);
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Empty,
                ..
            } => {
                self.finish_step(Value::nothing());
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Function { parameter, body },
                ..
            } => {
                self.finish_step(Value::Function { parameter, body });
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Identifier { name },
                ..
            } => {
                let mut value = Value::ProvidedFunction { name: name.clone() };

                for stack_frame in self.call_stack.iter().rev() {
                    if stack_frame.parameter == name {
                        value = stack_frame.argument.clone();
                        break;
                    }
                }

                self.finish_step(value);
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Number { value },
                ..
            } => {
                self.finish_step(Value::Integer { value });
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Recursion,
                ..
            } => {
                let stack_frame =
                    self.call_stack.last().cloned().unwrap_or_else(|| {
                        StackFrame {
                            parameter: "".to_string(),
                            argument: Value::nothing(),
                            root: codebase.root().path,
                        }
                    });

                self.finish_step(Value::Function {
                    parameter: stack_frame.parameter,
                    body: stack_frame.root,
                });
            }
            EvalStep::Derived {
                step: DerivedEvalStep::Tuple,
                ..
            } => {
                let values = evaluated_children;
                self.finish_step(Value::Tuple { values });
            }
            EvalStep::Synthetic {
                step: SyntheticEvalStep::PopStackFrame,
            } => {
                self.call_stack.pop();
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

    fn finish_step(&mut self, output: Value) {
        // When this is called, the current step has already been removed from
        // the stack.

        self.state = if self.eval_stack.last().is_some() {
            self.evaluated_children.push(output);
            RuntimeState::Running
        } else {
            RuntimeState::Finished { output }
        };
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }
}

#[derive(Clone, Debug)]
struct StackFrame {
    parameter: String,
    argument: Value,
    root: NodePath,
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Apply, Body, Codebase, Function, NodePath, SyntaxNode},
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
            let node = Function::empty(change_set.nodes)
                .into_syntax_node(change_set.nodes);
            let hash = change_set.nodes.insert(node);

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(hash),
            );
        });

        let [expected_parameter, expected_body] =
            codebase.root().expect_children(codebase.nodes());

        let mut evaluator = Evaluator::default();
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
            let root = {
                let node = Body::empty()
                    .with_children([Apply::default()
                        .with_expression(SyntaxNode::Recursion)
                        .with_argument(
                            Body::default().into_syntax_node(change_set.nodes),
                        )
                        .into_syntax_node(change_set.nodes)])
                    .into_syntax_node(change_set.nodes);

                change_set.nodes.insert(node)
            };

            change_set.replace(
                &change_set.root_before_change(),
                &NodePath::for_root(root),
            )
        });

        let mut evaluator = Evaluator::default();
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
