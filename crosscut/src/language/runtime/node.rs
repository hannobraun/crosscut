use crate::language::code::{
    Body, Expression, NodePath, Nodes, SyntaxNode, TypedNode,
};

use super::Value;

#[derive(Clone, Debug)]
pub struct RuntimeNode {
    pub path: Option<NodePath>,
    pub kind: RuntimeNodeKind,
}

impl RuntimeNode {
    pub fn new(path: NodePath, nodes: &Nodes) -> Self {
        let kind = RuntimeNodeKind::new(path.clone(), nodes);

        Self {
            path: Some(path),
            kind,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeNodeKind {
    Apply {
        /// # The path of the apply node
        ///
        /// ## Implementation Node
        ///
        /// It's weird to have this field here, when there's already a `path`
        /// field in `RuntimeNode`. But this field is available always (and
        /// needed always), while the other one is optional.
        ///
        /// There are ways to avoid this. Like adding a path field to every
        /// variant that is constructed from a `SyntaxNode`, instead of having
        /// the one in `RuntimeNode`. But that seems error-prone, as that would
        /// have to be done correctly for every new variant.
        ///
        /// Or `RuntimeNode` could become an enum that distinguishes between
        /// runtime nodes created from `SyntaxNode`, or runtime nodes created
        /// synthetically. But that seems overly complicated.
        ///
        /// In the end, this seems like a decent compromise, while the runtime
        /// still has its current shape. Long-term, it will probably become
        /// lower-level, more like a bytecode interpreter.
        path: NodePath,

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
    PopStackFrame {
        output: Value,
    },
    Recursion,
    Tuple {
        to_evaluate: Vec<NodePath>,
        evaluated: Vec<Value>,
    },
}

impl RuntimeNodeKind {
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
                    path,
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
