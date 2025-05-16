use crate::language::code::{Codebase, NodePath, SiblingIndex, SyntaxNode};

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
        values_to_evaluate: Vec<NodePath>,
        evaluated_values: Vec<Value>,
    },
    Generic {
        path: NodePath,
        evaluated_children: Vec<Value>,
    },
}

impl RuntimeNode {
    pub fn new(path: NodePath, codebase: &Codebase) -> Self {
        let syntax_node = codebase.node_at(&path);

        match syntax_node.node {
            SyntaxNode::Apply {
                expression,
                argument,
            } => Self::Apply {
                path: path.clone(),
                expression: RuntimeChild::Unevaluated {
                    path: NodePath::new(
                        *expression,
                        Some((path.clone(), SiblingIndex { index: 0 })),
                        codebase.nodes(),
                    ),
                },
                argument: RuntimeChild::Unevaluated {
                    path: NodePath::new(
                        *argument,
                        Some((path, SiblingIndex { index: 1 })),
                        codebase.nodes(),
                    ),
                },
            },
            SyntaxNode::Empty => Self::Empty,
            SyntaxNode::Function { parameter: _, body } => Self::Function {
                body: NodePath::new(
                    *body,
                    Some((path, SiblingIndex { index: 1 })),
                    codebase.nodes(),
                ),
            },
            SyntaxNode::Identifier { name } => {
                Self::Identifier { name: name.clone() }
            }
            SyntaxNode::Number { value } => Self::Number { value: *value },
            SyntaxNode::Recursion => Self::Recursion,
            SyntaxNode::Tuple { values, .. } => Self::Tuple {
                values_to_evaluate: values
                    .inner
                    .iter()
                    .copied()
                    .enumerate()
                    .rev()
                    .map(|(index, hash)| {
                        NodePath::new(
                            hash,
                            Some((path.clone(), SiblingIndex { index })),
                            codebase.nodes(),
                        )
                    })
                    .collect(),
                evaluated_values: Vec::new(),
            },
            _ => {
                let evaluated_children = Vec::new();

                Self::Generic {
                    path,
                    evaluated_children,
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

            Self::Tuple {
                evaluated_values, ..
            } => {
                evaluated_values.push(value);
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

            Self::Generic {
                evaluated_children, ..
            } => {
                evaluated_children.push(value);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeChild {
    Unevaluated { path: NodePath },
    Evaluated { value: Value },
}
