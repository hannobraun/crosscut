use crate::language::code::{
    Apply, Body, ChildIndex, Function, NodePath, Nodes, SyntaxNode,
};

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
            SyntaxNode::Body { children, add } => {
                let body = Body {
                    children: children.clone(),
                    add: *add,
                };

                let to_evaluate =
                    body.children().to_paths(&path, nodes).rev().collect();
                let evaluated = Vec::new();

                Self::Body {
                    to_evaluate,
                    evaluated,
                }
            }
            SyntaxNode::Empty => Self::Empty,
            SyntaxNode::Function { parameter, body } => {
                let function = Function::new(parameter, *body, nodes);
                let body = function.body().into_path(path, nodes);

                Self::Function {
                    parameter: function.parameter.name,
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
