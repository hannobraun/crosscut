use crate::language::code::{
    Codebase, Expression, IntrinsicFunction, Literal, NodePath, Type,
};

use super::{Effect, RuntimeState, Value, ValueWithSource};

#[derive(Clone, Debug)]
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
    pub fn evaluate(
        &mut self,
        expression: &Expression,
        path: NodePath,
        codebase: &Codebase,
    ) -> EvaluateUpdate {
        match expression {
            Expression::HostFunction { id } => {
                return EvaluateUpdate::UpdateState {
                    new_state: RuntimeState::Effect {
                        effect: Effect::ApplyHostFunction {
                            id: *id,
                            input: self.active_value.inner.clone(),
                        },
                        path,
                    },
                };
            }
            Expression::IntrinsicFunction { intrinsic } => {
                match intrinsic {
                    IntrinsicFunction::Identity => {
                        // Active value stays the same.
                    }
                    IntrinsicFunction::Literal { literal } => {
                        let Value::Nothing = self.active_value.inner else {
                            // A literal is a function that takes `None`. If
                            // that isn't what we currently have, that's an
                            // error.

                            return EvaluateUpdate::UpdateState {
                                new_state: RuntimeState::Effect {
                                    effect: Effect::UnexpectedInput {
                                        expected: Type::Nothing,
                                        actual: self.active_value.inner.clone(),
                                    },
                                    path,
                                },
                            };
                        };

                        let value = {
                            match *literal {
                                Literal::Function => {
                                    let Some(child) = codebase.child_of(&path)
                                    else {
                                        unreachable!(
                                            "Function literal must have a \
                                            child, or it wouldn't have been \
                                            resolved as a function literal."
                                        );
                                    };

                                    Value::Function {
                                        body: *child.hash(),
                                    }
                                }
                                Literal::Integer { value } => {
                                    Value::Integer { value }
                                }
                                Literal::Tuple => {
                                    let Some(child) = codebase.child_of(&path)
                                    else {
                                        unreachable!(
                                            "Tuple literal must have a child, \
                                            or it wouldn't have been resolved \
                                            as a tuple literal."
                                        );
                                    };

                                    self.active_value = ValueWithSource {
                                        inner: Value::Tuple {
                                            elements: Vec::new(),
                                        },
                                        source: Some(path),
                                    };
                                    self.advance();

                                    return EvaluateUpdate::NewContext {
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
            }
        };

        self.advance();

        EvaluateUpdate::UpdateState {
            new_state: RuntimeState::Running {
                active_value: self.active_value.clone(),
            },
        }
    }

    pub fn advance(&mut self) {
        self.nodes_from_root.pop();
    }
}

pub enum EvaluateUpdate {
    UpdateState { new_state: RuntimeState },
    NewContext { root: NodePath, active_value: Value },
}
