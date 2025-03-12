use crate::language::code::{
    Codebase, IntrinsicFunction, Literal, NodePath, Type,
};

use super::{Effect, RuntimeState, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    pub next: Option<ContextNode>,
    pub active_value: Value,
}

impl Context {
    pub fn evaluate_intrinsic_function(
        &mut self,
        intrinsic: &IntrinsicFunction,
        path: NodePath,
        codebase: &Codebase,
    ) -> EvaluateUpdate {
        match intrinsic {
            IntrinsicFunction::Drop => {
                self.active_value = Value::Nothing;
            }
            IntrinsicFunction::Eval => {
                let Value::Function { body } = self.active_value else {
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
                let Value::Nothing = self.active_value else {
                    return self.unexpected_input(Type::Nothing, path);
                };

                let value = {
                    match *literal {
                        Literal::Function => {
                            let node = codebase.node_at(path);
                            let mut children = node.children(codebase.nodes());

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
                                body: child.path.hash,
                            }
                        }
                        Literal::Integer { value } => Value::Integer { value },
                        Literal::Tuple => {
                            let node = codebase.node_at(path);
                            let mut children = node.children(codebase.nodes());

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

                            self.active_value = Value::Tuple {
                                elements: Vec::new(),
                            };
                            self.advance();

                            return EvaluateUpdate::PushContext {
                                root: child.path,
                                active_value: Value::Nothing,
                            };
                        }
                    }
                };

                self.active_value = value;
            }
        }

        self.advance();

        EvaluateUpdate::UpdateState {
            new_state: RuntimeState::Running {
                active_value: self.active_value.clone(),
                path: Some(path),
            },
        }
    }

    pub fn advance(&mut self) {
        self.next = self
            .next
            .take()
            .and_then(|next| next.parent.map(|child| *child));
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
                    actual: self.active_value.clone(),
                },
                path,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextNode {
    pub syntax_node: NodePath,
    pub parent: Option<Box<ContextNode>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum EvaluateUpdate {
    UpdateState { new_state: RuntimeState },
    PushContext { root: NodePath, active_value: Value },
}
