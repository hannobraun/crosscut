use crate::language::code::{NodePath, Nodes, SiblingIndex, SyntaxNode};

use super::Value;

#[derive(Clone, Debug)]
pub enum RuntimeNode {
    Apply {
        path: NodePath,
        expression: RuntimeChild,
        argument: RuntimeChild,
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
        values_to_evaluate: Vec<NodePath>,
        evaluated_values: Vec<Value>,
    },
}

impl RuntimeNode {
    pub fn new(path: NodePath, nodes: &Nodes) -> Self {
        match nodes.get(path.hash()) {
            SyntaxNode::Apply {
                expression,
                argument,
            } => {
                let expression = RuntimeChild::Unevaluated {
                    path: NodePath::new(
                        expression.hash,
                        Some((path.clone(), SiblingIndex { index: 0 })),
                        nodes,
                    ),
                };
                let argument = RuntimeChild::Unevaluated {
                    path: NodePath::new(
                        *argument,
                        Some((path.clone(), SiblingIndex { index: 1 })),
                        nodes,
                    ),
                };

                Self::Apply {
                    path,
                    expression,
                    argument,
                }
            }
            SyntaxNode::Empty => Self::Empty,
            SyntaxNode::Function { parameter, body } => {
                let parameter = {
                    let parameter = nodes.get(parameter);
                    let SyntaxNode::Binding { name } = parameter else {
                        panic!(
                            "Expected parameter of function to be a binding:\n\
                            {parameter:#?}"
                        );
                    };

                    name.clone()
                };
                let body = body.inner.first().unwrap();
                let body = NodePath::new(
                    *body,
                    Some((path, SiblingIndex { index: 1 })),
                    nodes,
                );

                Self::Function { parameter, body }
            }
            SyntaxNode::Identifier { name } => {
                let name = name.clone();

                Self::Identifier { name }
            }
            SyntaxNode::Number { value } => {
                let value = *value;

                Self::Number { value }
            }
            SyntaxNode::Recursion => Self::Recursion,
            SyntaxNode::Tuple { values, .. } => {
                let values_to_evaluate = values
                    .inner
                    .iter()
                    .copied()
                    .enumerate()
                    .rev()
                    .map(|(index, hash)| {
                        NodePath::new(
                            hash,
                            Some((path.clone(), SiblingIndex { index })),
                            nodes,
                        )
                    })
                    .collect();
                let evaluated_values = Vec::new();

                Self::Tuple {
                    values_to_evaluate,
                    evaluated_values,
                }
            }
            syntax_node => {
                // For the most part, this would only happen if there's a bug in
                // the compiler or evaluator. This still shouldn't be an
                // `unreachable!` though, as it's also a possible consequence of
                // somebody messing with the store code database.
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

            Self::Tuple {
                evaluated_values, ..
            } => {
                evaluated_values.push(value);
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
