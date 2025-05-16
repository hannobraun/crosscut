use crate::language::code::{Codebase, NodePath, SiblingIndex, SyntaxNode};

use super::Value;

#[derive(Clone, Debug)]
pub enum RuntimeNode {
    Apply {
        path: NodePath,
        expression: RuntimeChild,
        argument: RuntimeChild,
    },
    Generic {
        path: NodePath,
        children_to_evaluate: Vec<NodePath>,
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
            _ => {
                let children_to_evaluate = syntax_node
                    .inputs(codebase.nodes())
                    .map(|located_node| located_node.path)
                    .rev()
                    .collect();
                let evaluated_children = Vec::new();

                Self::Generic {
                    path,
                    children_to_evaluate,
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

            Self::Apply {
                expression: RuntimeChild::Evaluated { .. },
                argument: RuntimeChild::Evaluated { .. },
                ..
            } => {
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
