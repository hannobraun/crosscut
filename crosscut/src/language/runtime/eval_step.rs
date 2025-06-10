use std::{array, collections::VecDeque};

use crate::language::code::{
    Body, Expression, NodePath, Nodes, SyntaxNode, TypedNode,
};

use super::Value;

#[derive(Clone, Debug)]
pub enum EvalStep {
    /// # An evaluation step that was derived from a syntax node
    Derived {
        path: NodePath,
        step: DerivedEvalStep,
    },

    /// # An evaluation step that has no equivalent in the syntax tree
    Synthetic { step: SyntheticEvalStep },
}

impl EvalStep {
    pub fn derived(
        path: NodePath,
        eval_queue: &mut VecDeque<ChildToEvaluate>,
        nodes: &Nodes,
    ) -> Self {
        let step = DerivedEvalStep::new(path.clone(), eval_queue, nodes);
        Self::Derived { path, step }
    }
}

#[derive(Clone, Debug)]
pub enum DerivedEvalStep {
    Apply {
        expression: RuntimeChild,
        argument: RuntimeChild,
        is_tail_call: bool,
    },
    Body {
        to_evaluate: Vec<NodePath>,
        evaluated: Vec<Value>,
    },
    Empty,
    Function {
        parameter: String,
        body: NodePath,
    },
    Identifier {
        name: String,
    },
    Number {
        value: i32,
    },
    Recursion,
    Tuple {
        to_evaluate: Vec<NodePath>,
        evaluated: Vec<Value>,
    },
}

impl DerivedEvalStep {
    pub fn new(
        path: NodePath,
        eval_queue: &mut VecDeque<ChildToEvaluate>,
        nodes: &Nodes,
    ) -> Self {
        let TypedNode::Expression { expression } =
            TypedNode::from_hash(path.hash(), nodes)
        else {
            // For the most part, this would only happen if there's a bug in the
            // compiler or evaluator. This still shouldn't be an `unreachable!`
            // though, as it's also a possible consequence of somebody messing
            // with the stored code database.
            panic!("Expected expression.");
        };

        match expression {
            Expression::Apply { apply } => {
                // The reverted order of arguments is deliberate. It's required
                // to make the queue work correctly.
                for child in [apply.argument(), apply.expression()] {
                    eval_queue.push_front(ChildToEvaluate {
                        path: child.into_path(path.clone(), nodes),
                    });
                }

                let [expression, argument] =
                    array::from_fn(|_| RuntimeChild::Unevaluated);

                let is_tail_call =
                    if let Some((parent_path, child_index)) = path.parent() {
                        if let SyntaxNode::Body { children, .. } =
                            nodes.get(parent_path.hash())
                        {
                            child_index.index + 1 == children.len()
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                Self::Apply {
                    expression,
                    argument,
                    is_tail_call,
                }
            }
            Expression::Body { body } => {
                let to_evaluate =
                    body.children().to_paths(&path, nodes).rev().collect();
                let evaluated = Vec::new();

                Self::Body {
                    to_evaluate,
                    evaluated,
                }
            }
            Expression::Empty => Self::Empty,
            Expression::Function { function } => {
                let body = function.body().into_path(path, nodes);
                let parameter = function.parameter.name;

                Self::Function { parameter, body }
            }
            Expression::Identifier { name } => {
                Self::Identifier { name: name.clone() }
            }
            Expression::Number { value } => Self::Number { value },
            Expression::Recursion => Self::Recursion,
            Expression::Tuple { tuple } => {
                let values = Body::from_hash(&tuple.values, nodes);
                let parent = tuple.values().into_path(path, nodes);

                let to_evaluate =
                    values.children().to_paths(&parent, nodes).rev().collect();
                let evaluated = Vec::new();

                Self::Tuple {
                    to_evaluate,
                    evaluated,
                }
            }
        }
    }

    pub fn child_was_evaluated(&mut self, value: Value) {
        match self {
            Self::Apply {
                expression: child @ RuntimeChild::Unevaluated,
                ..
            }
            | Self::Apply {
                expression: RuntimeChild::Evaluated { .. },
                argument: child @ RuntimeChild::Unevaluated,
                ..
            } => {
                *child = RuntimeChild::Evaluated { value };
            }

            Self::Body { evaluated, .. } | Self::Tuple { evaluated, .. } => {
                evaluated.push(value);
            }

            Self::Apply {
                expression: RuntimeChild::Evaluated { .. },
                argument: RuntimeChild::Evaluated { .. },
                ..
            }
            | Self::Empty
            | Self::Function { .. }
            | Self::Identifier { .. }
            | Self::Number { .. }
            | Self::Recursion => {
                unreachable!("Node has no unevaluated children: {self:#?}")
            }
        }
    }
}

#[derive(Debug)]
pub struct ChildToEvaluate {
    pub path: NodePath,
}

#[derive(Clone, Debug)]
pub enum RuntimeChild {
    Unevaluated,
    Evaluated { value: Value },
}

#[derive(Clone, Debug)]
pub enum SyntheticEvalStep {
    PopStackFrame { output: Value },
}

impl SyntheticEvalStep {
    pub fn child_was_evaluated(&mut self, value: Value) {
        match self {
            Self::PopStackFrame { output } => {
                *output = value;
            }
        }
    }
}
