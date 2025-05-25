use crate::language::code::{Expression, NodePath, Nodes, TypedNode};

use super::Value;

#[derive(Clone, Debug)]
pub enum RuntimeNode {
    Apply {
        path: NodePath,
        expression: RuntimeChild,
        argument: RuntimeChild,
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
    PopStackFrame {
        output: Value,
    },
    Recursion,
    Tuple {
        to_evaluate: Vec<NodePath>,
        evaluated: Vec<Value>,
    },
}

impl RuntimeNode {
    pub fn new(path: NodePath, nodes: &Nodes) -> Self {
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
                let [expression, argument] =
                    [apply.expression(), apply.argument()].map(|child| {
                        RuntimeChild::Unevaluated {
                            path: child.into_path(path.clone(), nodes),
                        }
                    });

                Self::Apply {
                    path,
                    expression,
                    argument,
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
                let parent = path;

                let to_evaluate =
                    tuple.values().to_paths(&parent, nodes).rev().collect();
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
                expression: child @ RuntimeChild::Unevaluated { .. },
                ..
            }
            | Self::Apply {
                expression: RuntimeChild::Evaluated { .. },
                argument: child @ RuntimeChild::Unevaluated { .. },
                ..
            } => {
                *child = RuntimeChild::Evaluated { value };
            }

            Self::Body { evaluated, .. } | Self::Tuple { evaluated, .. } => {
                evaluated.push(value);
            }

            Self::PopStackFrame { output } => {
                *output = value;
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

#[derive(Clone, Debug)]
pub enum RuntimeChild {
    Unevaluated { path: NodePath },
    Evaluated { value: Value },
}
