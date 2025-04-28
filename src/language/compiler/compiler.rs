use crate::language::{
    code::{Children, Codebase, NodePath},
    packages::Packages,
};

use super::{replace::replace_node_and_update_parents, token::Token};

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
        self.codebase.make_change(|change_set| {
            let mut siblings =
                change_set.nodes.get(parent.hash()).to_children();
            let sibling_index = siblings.next_index();

            let token = Token {
                text: child_token,
                children: Children::new([]),
            };
            let child =
                token.compile(change_set.nodes, change_set.errors, packages);

            siblings.add(child);

            let token = change_set.nodes.get(parent.hash()).to_token(packages);
            let replacement = Token {
                text: &token,
                children: siblings.clone(),
            }
            .compile(change_set.nodes, change_set.errors, packages);
            let parent_path = replace_node_and_update_parents(
                parent,
                replacement,
                change_set,
                packages,
            );

            NodePath::new(
                child,
                Some(parent_path),
                sibling_index,
                change_set.nodes,
            )
        })
    }

    pub fn insert_parent(
        &mut self,
        child: &NodePath,
        parent_token: &str,
        packages: &Packages,
    ) -> NodePath {
        let children = Children::from([*child.hash()]);
        self.replace_inner(child, parent_token, children, packages)
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
        children: Children,
        packages: &Packages,
    ) -> NodePath {
        self.codebase.make_change(|change_set| {
            let replacement = Token {
                text: replacement_token,
                children: children.clone(),
            }
            .compile(change_set.nodes, change_set.errors, packages);

            replace_node_and_update_parents(
                to_replace.clone(),
                replacement,
                change_set,
                packages,
            )
        })
    }
}
