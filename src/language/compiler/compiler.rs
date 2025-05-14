use crate::language::{
    code::{Codebase, NodePath, SyntaxNode},
    packages::Packages,
};

use super::{expression, replace::replace_node_and_update_parents};

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
            let child = expression::compile(
                child_token,
                change_set.nodes,
                change_set.errors,
                packages,
            );

            let (parent_path, sibling_index) = {
                let mut node = change_set.nodes.get(parent.hash()).clone();

                let sibling_index = match &mut node {
                    SyntaxNode::AddValue
                    | SyntaxNode::Apply { .. }
                    | SyntaxNode::Empty
                    | SyntaxNode::Function { .. }
                    | SyntaxNode::Number { .. }
                    | SyntaxNode::Pattern { .. }
                    | SyntaxNode::ProvidedFunction { .. }
                    | SyntaxNode::Recursion
                    | SyntaxNode::UnresolvedIdentifier { .. } => {
                        panic!(
                            "Can't add child to this node:\n\
                            {node:#?}"
                        );
                    }

                    SyntaxNode::Tuple {
                        values: children, ..
                    }
                    | SyntaxNode::Test { children, .. } => children.add(child),
                };

                let hash = change_set.nodes.insert(node);

                // Adding a child doesn't change anything that could affect an
                // error on the parent. So we need to preserve that.
                if let Some(error) = change_set.errors.get(parent.hash()) {
                    change_set.errors.insert(hash, error.clone());
                }

                let path =
                    replace_node_and_update_parents(parent, hash, change_set);

                (path, sibling_index)
            };

            NodePath::new(
                child,
                Some((parent_path, sibling_index)),
                change_set.nodes,
            )
        })
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        self.replace_inner(to_replace, replacement_token, packages)
    }

    fn replace_inner(
        &mut self,
        to_replace: &NodePath,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath {
        self.codebase.make_change(|change_set| {
            let replacement = expression::compile(
                replacement_token,
                change_set.nodes,
                change_set.errors,
                packages,
            );

            replace_node_and_update_parents(
                to_replace.clone(),
                replacement,
                change_set,
            )
        })
    }
}
