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
        // In principle, tt should be possible to merge these two change sets
        // into a single one. In practice, that fails because we expect `root`
        // to be current for the second change set, but the changes in the first
        // message that up.
        //
        // This is not easy to fix. Due to lifetime issues, we can't read the
        // root from within the change set.
        //
        // However, the only reason the root is needed in the first place, is to
        // find the parents, for the replacement operation. And soon, this is
        // not going to be required any more, because it will be possible to
        // read the parent from `NodePath`.
        //
        // Once that change has been made, it should be straight-forward to
        // unify these change sets.

        let placeholder = self.codebase.make_change(|change_set| {
            let child = change_set.add(Node::new(NodeKind::Empty, []));

            let updated_parent = {
                let mut node = change_set.nodes().get(parent.hash()).clone();
                node.children_mut().add([child]);

                NodePath::new(
                    change_set.add(node),
                    parent.parent.clone().map(|parent| *parent),
                )
            };
            change_set.replace(&parent, &updated_parent);

            NodePath::new(child, Some(updated_parent))
        });

        let children = []; // just created this node with no children
        let root = self.codebase.root().path;
        self.codebase.make_change_with_errors(|change_set, errors| {
            replace_node_and_update_parents(
                &placeholder,
                child_token,
                children.into(),
                packages,
                root,
                change_set,
                errors,
            )
        })
    }

    pub fn insert_parent(
        &mut self,
        child: &NodePath,
        parent_token: &str,
        packages: &Packages,
    ) -> NodePath {
        self.replace_inner(child, parent_token, [*child.hash()], packages)
    }

    pub fn insert_sibling(
        &mut self,
        existing_sibling: &NodePath,
        new_sibling_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let parent = self
            .codebase()
            .parent_of(existing_sibling)
            .map(|located_node| located_node.path)
            .unwrap_or_else(|| {
                // The node we're adding a sibling for has no parent, meaning it
                // is the root of the syntax tree.
                //
                // The syntax tree always needs a single root. So we can't add a
                // sibling to the root node, without a new root node that can
                // serve as both of their parent.
                //
                // Adding this new root node is what we're doing here.
                self.insert_parent(existing_sibling, "", packages)
            });

        self.insert_child(parent, new_sibling_token, packages)
    }

    pub fn remove(
        &mut self,
        to_remove: &NodePath,
        to_update: &mut NodePath,
        packages: &Packages,
    ) {
        let mut update_stack = Vec::new();
        let mut path_to_update = to_update.clone();

        let need_to_update = loop {
            let parent = path_to_update.parent.clone();

            update_stack.push(path_to_update);

            if let Some(parent) = parent {
                let parent = *parent;

                if &parent == to_remove {
                    break true;
                } else {
                    path_to_update = parent;
                    continue;
                }
            } else {
                break false;
            }
        };

        let node_to_remove = self.codebase.nodes().get(to_remove.hash());

        let parent = if let Some(parent) = self.codebase.parent_of(to_remove) {
            // The node we're removing has a parent. We need to remove the
            // reference from that parent to the node.

            let mut children = parent.node.children().clone();
            children.replace(
                to_remove.hash(),
                node_to_remove.children().iter().copied(),
            );

            let parent = self.replace_inner(
                &parent.path,
                &parent.node.to_token(packages),
                children,
                packages,
            );

            Some(parent)
        } else {
            self.codebase.make_change(|change_set| {
                change_set.remove(to_remove);
            });

            None
        };

        if need_to_update {
            let mut parent = parent;

            while let Some(path) = update_stack.pop() {
                *to_update = NodePath::new(*to_update.hash(), parent.clone());

                parent = Some(NodePath::new(*path.hash(), parent));
            }
        }
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let children =
            self.codebase.node_at(to_replace).node.children().clone();
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
    let mut next_to_replace = to_replace.clone();

    let mut next_token = replacement_token.to_string();
    let mut next_children = children;

    let mut previous_replacement;
    let mut added_nodes = Vec::new();

    loop {
        let (node, maybe_error) =
            compile_token(&next_token, next_children, change_set, packages);

        let hash = change_set.add(node);
        previous_replacement = hash;

        added_nodes.push((next_to_replace.clone(), hash, maybe_error));

        if let Some(parent_path) = SyntaxTree::from_root(root.clone())
            .find_parent_of(&next_to_replace, change_set.nodes())
        {
            let parent_node = change_set.nodes().get(parent_path.hash());

            next_token = parent_node.to_token(packages);
            next_children = parent_node.children().clone();

            next_children
                .replace(next_to_replace.hash(), [previous_replacement]);

            next_to_replace = parent_path;

            continue;
        } else {
            break;
        };
    }

    let mut initial_replacement = None;
    let mut parent = None;

    while let Some((replaced, hash, maybe_error)) = added_nodes.pop() {
        let path = NodePath::new(hash, parent);
        parent = Some(path.clone());

        change_set.replace(&replaced, &path);

        initial_replacement = Some(path.clone());

        if let Some(error) = maybe_error {
            errors.insert(path, error);
        }
    }

    let Some(path) = initial_replacement else {
        unreachable!(
            "The loop above is executed at least once. The variable must have \
            been set."
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
