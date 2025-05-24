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
        match TypedNode::from_syntax_node(nodes.get(path.hash()), nodes) {
            TypedNode::Expression {
                expression: Expression::Apply { apply },
            } => {
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
            TypedNode::Expression {
                expression: Expression::Body { body },
            } => {
                let to_evaluate =
                    body.children().to_paths(&path, nodes).rev().collect();
                let evaluated = Vec::new();

                Self::Body {
                    to_evaluate,
                    evaluated,
                }
            }
            TypedNode::Expression {
                expression: Expression::Empty,
            } => Self::Empty,
            TypedNode::Expression {
                expression: Expression::Function { function },
            } => {
                let body = function.body().into_path(path, nodes);

                Self::Function {
                    parameter: function.parameter.name,
                    body,
                }
            }
            TypedNode::Expression {
                expression: Expression::Identifier { name },
            } => {
                let name = name.clone();

                Self::Identifier { name }
            }
            TypedNode::Expression {
                expression: Expression::Number { value },
            } => Self::Number { value },
            TypedNode::Expression {
                expression: Expression::Recursion,
            } => Self::Recursion,
            TypedNode::Expression {
                expression: Expression::Tuple { tuple },
            } => {
                let to_evaluate =
                    tuple.values().to_paths(&path, nodes).rev().collect();
                let evaluated = Vec::new();

                Self::Tuple {
                    to_evaluate,
                    evaluated,
                }
            }
            syntax_node => {
                // For the most part, this would only happen if there's a bug in
                // the compiler or evaluator. This still shouldn't be an
                // `unreachable!` though, as it's also a possible consequence of
                // somebody messing with the stored code database.
                panic!(
                    "Could not construct runtime node from syntax node: \n\
                    {syntax_node:#?}"
                );
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
