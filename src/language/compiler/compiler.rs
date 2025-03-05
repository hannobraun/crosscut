use crate::language::{
    code::{
        Children, CodeError, Codebase, Expression, IntrinsicFunction, Literal,
        Node, NodeKind, NodePath, SyntaxTree,
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

    pub fn codebase(&self) -> &Codebase {
        self.codebase
    }

    pub fn insert_child(
        &mut self,
        parent: NodePath,
        child_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let child = {
            let placeholder = self.codebase.make_change(|change_set| {
                let child = change_set.add(Node::new(NodeKind::Empty, []));

                let mut updated_parent =
                    change_set.nodes().get(parent.hash()).clone();
                updated_parent.children_mut().add([child]);

                change_set.replace(parent, updated_parent);

                NodePath { hash: child }
            });

            self.replace(&placeholder, child_token, packages)
        };

        let Some(parent) = self.codebase.parent_of(&child) else {
            unreachable!(
                "Just inserted `child` as child of a parent. So a parent must \
                exist."
            );
        };

        self.replace(
            &parent,
            &self.codebase.node_at(&parent).display(packages).to_string(),
            packages,
        );

        child
    }

    pub fn insert_parent(
        &mut self,
        child: &NodePath,
        parent_token: &str,
        packages: &Packages,
    ) -> NodePath {
        self.replace_inner(child, parent_token, [child.hash], packages)
    }

    pub fn remove(&mut self, to_remove: NodePath) {
        let node_to_remove = self.codebase.nodes().get(to_remove.hash());

        if let Some(parent_path) = self.codebase.parent_of(&to_remove) {
            // The node we're removing has a parent. We need to remove the
            // reference from that parent to the node.

            let mut updated_parent =
                self.codebase.node_at(&parent_path).clone();
            updated_parent.children_mut().replace(
                to_remove.hash(),
                node_to_remove.children().iter().copied(),
            );

            replace_node(&parent_path, updated_parent, self.codebase);
        } else {
            self.codebase.make_change(|change_set| {
                change_set.remove(to_remove);
            });
        }
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let children = self.codebase.node_at(to_replace).children().clone();
        self.replace_inner(to_replace, replacement_token, children, packages)
    }

    fn replace_inner(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        children: impl Into<Children>,
        packages: &Packages,
    ) -> NodePath {
        let (node, maybe_error) = compile_token(
            replacement_token,
            children.into(),
            self.codebase,
            packages,
        );

        let path = replace_node(to_replace, node, self.codebase);
        if let Some(error) = maybe_error {
            self.codebase.insert_error(path, error);
        }

        path
    }
}

fn compile_token(
    token: &str,
    children: Children,
    codebase: &mut Codebase,
    packages: &Packages,
) -> (Node, Option<CodeError>) {
    let (node, maybe_error) = if token.is_empty() {
        let (kind, error) = if children.has_multiple().is_none() {
            (NodeKind::Empty, None)
        } else {
            (
                NodeKind::Error {
                    node: token.to_string(),
                },
                Some(CodeError::EmptyNodeWithMultipleChildren),
            )
        };

        (Node::new(kind, children), error)
    } else if let Some((node, maybe_err)) =
        resolve_keyword(token, &children, codebase)
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
    children: &Children,
    codebase: &mut Codebase,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "fn" => {
            // Every function must have a child. Other code assumes that.
            let children = if children.has_none() {
                let child = codebase.make_change(|change_set| {
                    change_set.add(Node::new(NodeKind::Empty, []))
                });
                Children::new(Some(child))
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
            Node::new(NodeKind::Recursion, children.iter().copied()),
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

fn replace_node(
    to_replace: &NodePath,
    replacement: Node,
    codebase: &mut Codebase,
) -> NodePath {
    let root = codebase.root().path;

    codebase.make_change(|change_set| {
        let mut next_to_replace = *to_replace;
        let mut next_replacement = replacement;

        let mut previous_replacement;
        let mut initial_replacement = None;

        loop {
            let path = change_set.replace(next_to_replace, next_replacement);

            initial_replacement = initial_replacement.or(Some(path));
            previous_replacement = path.hash;

            if let Some(parent) = SyntaxTree::from_root(root.hash)
                .find_parent_of(&next_to_replace.hash, change_set.nodes())
            {
                next_replacement =
                    change_set.nodes().get(parent.hash()).clone();
                next_replacement
                    .children_mut()
                    .replace(next_to_replace.hash(), [previous_replacement]);

                next_to_replace = parent;

                continue;
            } else {
                break;
            };
        }

        if let Some(path) = initial_replacement {
            path
        } else {
            unreachable!(
                "The loop above is executed at least once. The variable must \
                have been set."
            );
        }
    })
}
