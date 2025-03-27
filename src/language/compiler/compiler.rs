use crate::language::{
    code::{
        CandidateForResolution, Children, CodeError, Codebase, Errors, Literal,
        NewChangeSet, Node, NodeKind, NodePath,
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
        self.codebase.make_change_with_errors(|change_set, errors| {
            let (child, maybe_error) = compile_token(
                child_token,
                Children::new([]),
                change_set,
                packages,
            );
            let child = change_set.add(child);

            let mut siblings =
                change_set.nodes().get(parent.hash()).to_children();
            let sibling_index = siblings.add(child);

            let parent_path = replace_node_and_update_parents(
                &parent,
                &change_set.nodes().get(parent.hash()).to_token(packages),
                siblings,
                packages,
                change_set,
                errors,
            );

            let child_path = NodePath::new(
                child,
                Some(parent_path),
                sibling_index,
                change_set.nodes(),
            );

            if let Some(error) = maybe_error {
                errors.insert(child_path.clone(), error);
            }

            child_path
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
        let parent = existing_sibling.parent().cloned().unwrap_or_else(|| {
            // The node we're adding a sibling for has no parent, meaning it is
            // the root of the syntax tree.
            //
            // The syntax tree always needs a single root. So we can't add a
            // sibling to the root node, without a new root node that can serve
            // as both of their parent.
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

        let update_node_is_descendent = loop {
            let parent = path_to_update.parent().cloned();

            update_stack.push(path_to_update);

            if let Some(parent) = parent {
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

        let parent = if let Some(parent) = to_remove.parent() {
            // The node we're removing has a parent. We need to remove the
            // reference from that parent to the node.

            let parent = self.codebase.node_at(parent);

            let mut children = parent.node.to_children();
            children
                .replace(to_remove.hash(), node_to_remove.children().copied());

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

        let update_node_is_ancestor = to_update.is_ancestor_of(to_remove);
        let update_node_is_lateral_relation =
            !update_node_is_descendent && !update_node_is_ancestor;

        if update_node_is_descendent || update_node_is_lateral_relation {
            let to_update_new_sibling_index = if to_update.parent()
                == to_remove.parent()
                && to_update.sibling_index() > to_remove.sibling_index()
            {
                to_update.sibling_index() - 1
            } else {
                to_update.sibling_index()
            };

            let mut parent = if update_node_is_descendent {
                parent
            } else {
                update_stack.pop();
                Some(self.codebase.root().path)
            };

            while let Some(path) = update_stack.pop() {
                *to_update = NodePath::new(
                    *to_update.hash(),
                    parent.clone(),
                    to_update_new_sibling_index,
                    self.codebase.nodes(),
                );
                parent = Some(NodePath::new(
                    *path.hash(),
                    parent,
                    path.sibling_index(),
                    self.codebase.nodes(),
                ));
            }
        } else if update_node_is_ancestor {
            *to_update = self.codebase.latest_version_of(to_update);
        }
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let children = self.codebase.node_at(to_replace).node.to_children();
        self.replace_inner(to_replace, replacement_token, children, packages)
    }

    fn replace_inner(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        children: impl Into<Children>,
        packages: &Packages,
    ) -> NodePath {
        self.codebase.make_change_with_errors(|change_set, errors| {
            replace_node_and_update_parents(
                to_replace,
                replacement_token,
                children.into(),
                packages,
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

        if let Some(parent_path) = next_to_replace.parent().cloned() {
            let parent_node = change_set.nodes().get(parent_path.hash());

            next_token = parent_node.to_token(packages);
            next_children = parent_node.to_children();

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
        let path = NodePath::new(
            hash,
            parent,
            replaced.sibling_index(),
            change_set.nodes(),
        );
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
        if children.is_multiple_children().is_none() {
            (Node::new(NodeKind::Empty, children), None)
        } else {
            (
                Node::new(
                    NodeKind::Error {
                        node: token.to_string(),
                    },
                    children,
                ),
                Some(CodeError::EmptyNodeWithMultipleChildren),
            )
        }
    } else if let Some((node, maybe_err)) = resolve_keyword(token, &children) {
        (node, maybe_err)
    } else {
        match resolve_function(token, children, change_set, packages) {
            Ok(node) => (node, None),
            Err((children, candidates)) => (
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
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "self" => Some((
            Node::new(NodeKind::Recursion, children.iter().copied()),
            None,
        )),
        _ => None,
    }
}

fn resolve_function(
    name: &str,
    children: Children,
    change_set: &mut NewChangeSet,
    packages: &Packages,
) -> Result<Node, (Children, Vec<CandidateForResolution>)> {
    let provided_function = packages.resolve_function(name);
    let literal = resolve_literal(name);

    match (provided_function, literal) {
        (Some(id), None) => {
            Ok(Node::new(NodeKind::ProvidedFunction { id }, children))
        }
        (None, Some(literal)) => match literal {
            Literal::Function => {
                // Every function must have a child. Other code assumes that.
                let children = if children.is_empty() {
                    let child = change_set.add(Node::new(NodeKind::Empty, []));
                    Children::new(Some(child))
                } else {
                    children.clone()
                };

                Ok(Node::new(NodeKind::LiteralFunction, children))
            }
            Literal::Integer { value } => {
                Ok(Node::new(NodeKind::LiteralInteger { value }, children))
            }
            Literal::Tuple => Ok(Node::new(NodeKind::LiteralTuple, children)),
        },
        (None, None) => {
            let candidates = Vec::new();
            Err((children, candidates))
        }
        (provided_function, literal) => {
            let mut candidates = Vec::new();

            if let Some(id) = provided_function {
                candidates
                    .push(CandidateForResolution::ProvidedFunction { id });
            }
            if let Some(literal) = literal {
                candidates.push(CandidateForResolution::Literal { literal });
            }

            Err((children, candidates))
        }
    }
}

fn resolve_literal(name: &str) -> Option<Literal> {
    if let Ok(value) = name.parse() {
        Some(Literal::Integer { value })
    } else {
        match name {
            "fn" => Some(Literal::Function),
            "tuple" => Some(Literal::Tuple),
            _ => None,
        }
    }
}
