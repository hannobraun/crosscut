use crate::language::code::{
    Apply, ChildIndex, Expressions, Function, NodePath, Nodes, SyntaxNode,
};

use super::Value;

#[derive(Clone, Debug)]
pub enum RuntimeNode {
    Apply {
        path: NodePath,
        expression: RuntimeChild,
        argument: RuntimeChild,
    },
    Empty,
    Expressions {
        to_evaluate: Vec<NodePath>,
        evaluated: Vec<Value>,
    },
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
        match nodes.get(path.hash()) {
            SyntaxNode::Apply {
                expression,
                argument,
            } => {
                let apply = Apply {
                    expression: *expression,
                    argument: *argument,
                };

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
            SyntaxNode::Empty => Self::Empty,
            SyntaxNode::Expressions { expressions, add } => {
                let expressions = Expressions {
                    expressions: expressions.clone(),
                    add: *add,
                };

                let to_evaluate = expressions
                    .expressions()
                    .iter()
                    .map(|child| child.into_path(path.clone(), nodes))
                    .rev()
                    .collect();
                let evaluated = Vec::new();

                Self::Expressions {
                    to_evaluate,
                    evaluated,
                }
            }
            SyntaxNode::Function { parameter, body } => {
                let function = Function::new(parameter, body.clone(), nodes);

                let parameter = function.parameter;
                let body = function.body.first().unwrap();
                let body = NodePath::new(
                    *body,
                    Some((path, ChildIndex { index: 1 })),
                    nodes,
                );

                Self::Function {
                    parameter: parameter.name,
                    body,
                }
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
                let to_evaluate = values
                    .iter()
                    .copied()
                    .enumerate()
                    .rev()
                    .map(|(index, hash)| {
                        NodePath::new(
                            hash,
                            Some((path.clone(), ChildIndex { index })),
                            nodes,
                        )
                    })
                    .collect();
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

            Self::Expressions { evaluated, .. }
            | Self::Tuple { evaluated, .. } => {
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
