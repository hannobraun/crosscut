use crate::language::{
    code::{Children, Codebase, Errors, NewChangeSet, NodePath},
    packages::Packages,
};

use super::{
    strategy::{NodeAddedDuringReplacement, ReplacementStrategy},
    token::Token,
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
            let mut siblings =
                change_set.nodes().get(parent.hash()).to_children();
            let sibling_index = siblings.next_index();

            let (child, maybe_error) = Token {
                text: child_token,
                parent: Some(&parent),
                sibling_index,
                children: Children::new([]),
            }
            .compile(change_set.nodes(), packages);

            let child = change_set.add(child);
            siblings.add(child);

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
                errors.insert(*child_path.hash(), error);
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
            children.replace(to_remove.hash(), node_to_remove.to_children());

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
    let mut strategy = ReplacementStrategy {
        next_to_replace: to_replace.clone(),
        next_token: replacement_token.to_string(),
        next_children: children,
        added_nodes: Vec::new(),
    };

    loop {
        let token = strategy.next_action();
        let (node, maybe_error) = token.compile(change_set.nodes(), packages);

        let added = change_set.add(node);

        strategy.added_nodes.push(NodeAddedDuringReplacement {
            replaced: strategy.next_to_replace.clone(),
            added,
            maybe_error,
        });

        if let Some(parent_path) = strategy.next_to_replace.parent().cloned() {
            let parent_node = change_set.nodes().get(parent_path.hash());

            strategy.next_token = parent_node.to_token(packages);
            strategy.next_children = parent_node.to_children();

            strategy
                .next_children
                .replace(strategy.next_to_replace.hash(), [added]);

            strategy.next_to_replace = parent_path;

            continue;
        } else {
            break;
        };
    }

    let mut initial_replacement = None;
    let mut parent = None;

    while let Some(NodeAddedDuringReplacement {
        replaced,
        added,
        maybe_error,
    }) = strategy.added_nodes.pop()
    {
        let path = NodePath::new(
            added,
            parent,
            replaced.sibling_index(),
            change_set.nodes(),
        );
        parent = Some(path.clone());

        change_set.replace(&replaced, &path);

        initial_replacement = Some(path.clone());

        if let Some(error) = maybe_error {
            errors.insert(*path.hash(), error);
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
