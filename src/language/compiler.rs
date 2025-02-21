use std::ops::Deref;

use super::{
    code::{
        Children, CodeError, Codebase, Expression, IntrinsicFunction, Literal,
        Node, NodeKind, NodePath,
    },
    packages::Packages,
};

pub struct Compiler<'r> {
    codebase: &'r mut Codebase,
}

impl<'r> Compiler<'r> {
    pub fn new(codebase: &'r mut Codebase) -> Self {
        Self { codebase }
    }

    pub fn insert_parent(
        &mut self,
        child: &NodePath,
        parent: Node,
    ) -> NodePath {
        self.codebase.insert_node_as_parent(child, parent)
    }

    pub fn remove(&mut self, to_remove: &NodePath) {
        self.codebase.remove_node(to_remove);
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let mut path = *to_replace;

        let (node, maybe_error) = compile_token(
            replacement_token,
            &mut path,
            self.codebase,
            packages,
        );

        let path = self.codebase.replace_node(&path, node);
        if let Some(error) = maybe_error {
            self.codebase.insert_error(path, error);
        }

        path
    }
}

impl Deref for Compiler<'_> {
    type Target = Codebase;

    fn deref(&self) -> &Self::Target {
        self.codebase
    }
}

fn compile_token(
    token: &str,
    path: &mut NodePath,
    codebase: &mut Codebase,
    packages: &Packages,
) -> (Node, Option<CodeError>) {
    let node = codebase.node_at(path);
    let children = node.children().clone();

    let (node, maybe_error) = if token.is_empty() {
        (Node::new(NodeKind::Empty, children), None)
    } else if let Some((node, maybe_err)) =
        resolve_keyword(token, path, &children, codebase)
    {
        (node, maybe_err)
    } else {
        match resolve_function(token, packages) {
            Ok(expression) => (
                Node::new(NodeKind::Expression { expression }, children),
                None,
            ),
            Err(candidates) => (
                Node::new(
                    NodeKind::Error {
                        node: token.to_string(),
                    },
                    children,
                ),
                Some(CodeError::UnresolvedIdentifier { candidates }),
            ),
        }
    };

    (node, maybe_error)
}

fn resolve_keyword(
    name: &str,
    path: &mut NodePath,
    children: &Children,
    codebase: &mut Codebase,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "fn" => {
            // Every function must have a child. Other code assumes that.
            let children = if children.is_empty() {
                let child = codebase.insert_node_as_child(
                    path,
                    Node::new(NodeKind::Empty, None),
                );
                *path = codebase.latest_version_of(*path);

                Children::new(Some(*child.hash()))
            } else {
                children.clone()
            };

            Some((
                Node::new(
                    NodeKind::Expression {
                        expression: Expression::IntrinsicFunction {
                            intrinsic: IntrinsicFunction::Literal {
                                literal: Literal::Function,
                            },
                        },
                    },
                    children,
                ),
                None,
            ))
        }
        "self" => Some((
            Node::new(NodeKind::Recursion, children.into_iter().copied()),
            None,
        )),
        _ => None,
    }
}

fn resolve_function(
    name: &str,
    packages: &Packages,
) -> Result<Expression, Vec<Expression>> {
    let host_function = packages.resolve_function(name);
    let intrinsic_function = IntrinsicFunction::resolve(name);

    match (host_function, intrinsic_function) {
        (Some(id), None) => Ok(Expression::HostFunction { id }),
        (None, Some(intrinsic)) => {
            Ok(Expression::IntrinsicFunction { intrinsic })
        }
        (None, None) => {
            let candidates = Vec::new();
            Err(candidates)
        }
        (Some(id), Some(intrinsic)) => {
            let candidates = vec![
                Expression::HostFunction { id },
                Expression::IntrinsicFunction { intrinsic },
            ];
            Err(candidates)
        }
    }
}
