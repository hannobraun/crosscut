use crate::language::{
    code::{
        Children, CodeError, Codebase, Errors, Expression, IntrinsicFunction,
        Literal, NewChangeSet, Node, NodeKind, NodePath, SyntaxTree,
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
            &self.codebase.node_at(&parent).to_token(packages),
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

    pub fn remove(&mut self, to_remove: NodePath, packages: &Packages) {
        let node_to_remove = self.codebase.nodes().get(to_remove.hash());

        if let Some(parent_path) = self.codebase.parent_of(&to_remove) {
            // The node we're removing has a parent. We need to remove the
            // reference from that parent to the node.

            let parent_node = self.codebase.node_at(&parent_path).clone();

            let mut children = parent_node.children().clone();
            children.replace(
                to_remove.hash(),
                node_to_remove.children().iter().copied(),
            );

            self.replace_inner(
                &parent_path,
                &parent_node.to_token(packages),
                children,
                packages,
            );
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
        let root = self.codebase.root().path;
        self.codebase.make_change_with_errors(|change_set, errors| {
            replace_node_and_update_parents(
                to_replace,
                replacement_token,
                children.into(),
                packages,
                root,
                change_set,
                errors,
            )
        })
    }
}

fn replace_node_and_update_parents(
    to_replace: &NodePath,
    replacement_token: &str,
    children: Children,
    packages: &Packages,
    root: NodePath,
    change_set: &mut NewChangeSet,
    errors: &mut Errors,
) -> NodePath {
    let mut next_to_replace = *to_replace;
    let mut next_replacement;

    let mut next_token = replacement_token.to_string();
    let mut next_children = children;

    let mut previous_replacement;
    let mut initial_replacement = None;

    loop {
        let (node, maybe_error) =
            compile_token(&next_token, next_children, change_set, packages);

        let path = change_set.replace(next_to_replace, node);

        if let Some(error) = maybe_error {
            errors.insert(path, error);
        }

        initial_replacement = initial_replacement.or(Some(path));
        previous_replacement = path.hash;

        if let Some(parent_path) = SyntaxTree::from_root(root.hash)
            .find_parent_of(&next_to_replace.hash, change_set.nodes())
        {
            next_replacement =
                change_set.nodes().get(parent_path.hash()).clone();
            next_replacement
                .children_mut()
                .replace(next_to_replace.hash(), [previous_replacement]);

            next_token = next_replacement.to_token(packages);
            next_children = next_replacement.children().clone();

            next_to_replace = parent_path;

            continue;
        } else {
            break;
        };
    }

    let Some(path) = initial_replacement else {
        unreachable!(
            "The loop above is executed at least once. The variable \
                    must have been set."
        );
    };

    path
}

fn compile_token(
    token: &str,
    children: Children,
    change_set: &mut NewChangeSet,
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
        resolve_keyword(token, &children, change_set)
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
    change_set: &mut NewChangeSet,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "fn" => {
            // Every function must have a child. Other code assumes that.
            let children = if children.has_none() {
                let child = change_set.add(Node::new(NodeKind::Empty, []));
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
