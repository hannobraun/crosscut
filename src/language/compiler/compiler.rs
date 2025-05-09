use crate::language::{
    code::{Codebase, Expression, NodePath},
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
        parent: NodePath<Expression>,
        child_token: &str,
        packages: &Packages,
    ) -> NodePath<Expression> {
        self.codebase.make_change(|change_set| {
            let child = {
                expression::compile(
                    child_token,
                    change_set.nodes,
                    change_set.errors,
                    packages,
                )
            };

            let (parent_path, sibling_index) = {
                let mut node = change_set.nodes.get(parent.hash()).clone();

                let sibling_index = match &mut node {
                    Expression::Apply { .. }
                    | Expression::Empty
                    | Expression::Function { .. }
                    | Expression::Number { .. }
                    | Expression::ProvidedFunction { .. }
                    | Expression::Recursion
                    | Expression::UnresolvedIdentifier { .. } => {
                        panic!(
                            "Can't add child to this node:\n\
                            {node:#?}"
                        );
                    }

                    Expression::Tuple { values: children }
                    | Expression::Test { children, .. } => children.add(child),
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
                Some((parent_path.to_parent(), sibling_index)),
                Some(parent_path),
                change_set.nodes,
            )
        })
    }

    pub fn insert_parent(
        &mut self,
        child: &NodePath<Expression>,
        parent_token: &str,
        packages: &Packages,
    ) -> NodePath<Expression> {
        self.replace_inner(child, parent_token, packages)
    }

    pub fn insert_sibling(
        &mut self,
        existing_sibling: &NodePath<Expression>,
        new_sibling_token: &str,
        packages: &Packages,
    ) -> NodePath<Expression> {
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
        to_replace: &NodePath<Expression>,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath<Expression> {
        self.replace_inner(to_replace, replacement_token, packages)
    }

    fn replace_inner(
        &mut self,
        to_replace: &NodePath<Expression>,
        replacement_token: &str,
        packages: &Packages,
    ) -> NodePath<Expression> {
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
