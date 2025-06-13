use std::collections::VecDeque;

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
        children_to_evaluate: usize,
    },

    /// # An evaluation step that has no equivalent in the syntax tree
    Synthetic { step: SyntheticEvalStep },
}

impl EvalStep {
    pub fn derived(
        path: NodePath,
        eval_queue: &mut VecDeque<QueuedEvalStep>,
        nodes: &Nodes,
    ) -> Self {
        let queue_len_before = eval_queue.len();
        let step = DerivedEvalStep::new(path.clone(), eval_queue, nodes);
        let queue_len_after = eval_queue.len();

        let Some(num_children) = queue_len_after.checked_sub(queue_len_before)
        else {
            unreachable!(
                "Creating a derived eval step does not remove from the queue. \
                Subtracting the old length from the new must be valid."
            );
        };

        Self::Derived {
            path,
            step,
            children_to_evaluate: num_children,
        }
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
        evaluated_children: Vec<Value>,
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
        evaluated_children: Vec<Value>,
    },
}

impl DerivedEvalStep {
    pub fn new(
        path: NodePath,
        eval_queue: &mut VecDeque<QueuedEvalStep>,
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
                for child in apply.children().rev() {
                    eval_queue.push_front(QueuedEvalStep {
                        path: child.into_path(path.clone(), nodes),
                        parent: path.clone(),
                    });
                }

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
                    expression: RuntimeChild::Unevaluated,
                    argument: RuntimeChild::Unevaluated,
                    is_tail_call,
                }
            }
            Expression::Body { body } => {
                for child_path in body.children().to_paths(&path, nodes).rev() {
                    eval_queue.push_front(QueuedEvalStep {
                        path: child_path,
                        parent: path.clone(),
                    });
                }

                Self::Body {
                    evaluated_children: Vec::new(),
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
                let parent = tuple.values().into_path(path.clone(), nodes);

                for child_path in
                    values.children().to_paths(&parent, nodes).rev()
                {
                    eval_queue.push_front(QueuedEvalStep {
                        path: child_path,
                        parent: path.clone(),
                    });
                }

                let evaluated = Vec::new();

                Self::Tuple {
                    evaluated_children: evaluated,
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

            Self::Body {
                evaluated_children, ..
            }
            | Self::Tuple {
                evaluated_children, ..
            } => {
                evaluated_children.push(value);
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
pub struct QueuedEvalStep {
    pub path: NodePath,
    pub parent: NodePath,
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
