use crate::language::{
    code::{Codebase, IntrinsicFunction, Literal, NodePath, Type},
    packages::FunctionId,
};

use super::{Effect, RuntimeState, Value, ValueWithSource};

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl Context {
    pub fn evaluate_host_function(&self, id: FunctionId) -> Effect {
        Effect::ApplyHostFunction {
            id,
            input: self.active_value.inner.clone(),
        }
    }

    pub fn evaluate_intrinsic_function(
        &mut self,
        intrinsic: &IntrinsicFunction,
        path: NodePath,
        codebase: &Codebase,
    ) -> EvaluateUpdate {
        match intrinsic {
            IntrinsicFunction::Eval => {
                let Value::Function { body } = self.active_value.inner else {
                    return self.unexpected_input(Type::Function, path);
                };

                return EvaluateUpdate::PushContext {
                    root: NodePath { hash: body },
                    // Right now, the `eval` function doesn't support passing an
                    // argument to the function it evaluates.
                    active_value: Value::Nothing,
                };
            }
            IntrinsicFunction::Identity => {
                // Active value stays the same.
            }
            IntrinsicFunction::Literal { literal } => {
                let Value::Nothing = self.active_value.inner else {
                    return self.unexpected_input(Type::Nothing, path);
                };

                let value = {
                    match *literal {
                        Literal::Function => {
                            let mut children =
                                codebase.children_of(&path).to_paths();

                            let Some(child) = children.next() else {
                                unreachable!(
                                    "Function literal must have a child, or it \
                                    wouldn't have been resolved as a function \
                                    literal."
                                );
                            };

                            assert_eq!(
                                children.count(),
                                0,
                                "Only nodes with one child can be evaluated at \
                                this point.",
                            );

                            Value::Function {
                                body: *child.hash(),
                            }
                        }
                        Literal::Integer { value } => Value::Integer { value },
                        Literal::Tuple => {
                            let mut children =
                                codebase.children_of(&path).to_paths();

                            let Some(child) = children.next() else {
                                unreachable!(
                                    "Tuple literal must have a child, or it \
                                    wouldn't have been resolved as a tuple \
                                    literal."
                                );
                            };

                            assert_eq!(
                                children.count(),
                                0,
                                "Only nodes with one child can be evaluated at \
                                this point.",
                            );

                            self.active_value = ValueWithSource {
                                inner: Value::Tuple {
                                    elements: Vec::new(),
                                },
                                source: Some(path),
                            };
                            self.advance();

                            return EvaluateUpdate::PushContext {
                                root: child,
                                active_value: Value::Nothing,
                            };
                        }
                    }
                };

                self.active_value = ValueWithSource {
                    inner: value,
                    source: Some(path),
                };
            }
        }

        self.advance();

        EvaluateUpdate::UpdateState {
            new_state: RuntimeState::Running {
                active_value: self.active_value.inner.clone(),
                path: Some(path),
            },
        }
    }

    pub fn advance(&mut self) {
        self.nodes_from_root.pop();
    }

    fn unexpected_input(
        &self,
        expected: Type,
        path: NodePath,
    ) -> EvaluateUpdate {
        EvaluateUpdate::UpdateState {
            new_state: RuntimeState::Effect {
                effect: Effect::UnexpectedInput {
                    expected,
                    actual: self.active_value.inner.clone(),
                },
                path,
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EvaluateUpdate {
    UpdateState { new_state: RuntimeState },
    PushContext { root: NodePath, active_value: Value },
}
