use crate::language::{
    code::{Children, Errors, NewChangeSet, NodeHash, NodePath},
    packages::Packages,
};

use super::token::Token;

pub fn replace_node_and_update_parents(
    to_replace: NodePath,
    replacement_token: String,
    children: Children,
    change_set: &mut NewChangeSet,
    errors: &mut Errors,
    packages: &Packages,
) -> NodePath {
    let mut next_action = ReplaceAction::Start {
        to_replace,
        replacement_token,
        children,
    };

    loop {
        next_action = next_action.perform(change_set, errors, packages);

        if let ReplaceAction::Finish { path } = next_action {
            break path;
        }
    }
}

enum ReplaceAction {
    Start {
        to_replace: NodePath,
        replacement_token: String,
        children: Children,
    },
    CompileToken {
        path: NodePath,
        token: String,
        children: Children,
        replacements: Vec<Replacement>,
    },
    UpdatePath {
        replacement: Replacement,
        parent: Option<NodePath>,
        replacements: Vec<Replacement>,
    },
    Finish {
        path: NodePath,
    },
}

impl ReplaceAction {
    fn perform(
        self,
        change_set: &mut NewChangeSet,
        errors: &mut Errors,
        packages: &Packages,
    ) -> Self {
        match self {
            Self::Start {
                to_replace,
                replacement_token,
                children,
            } => {
                // comment added to force more readable formatting
                start(to_replace, replacement_token, children)
            }
            Self::CompileToken {
                path,
                token,
                children,
                replacements,
            } => {
                // comment added to force more readable formatting
                compile_token(
                    path,
                    token,
                    children,
                    replacements,
                    change_set,
                    errors,
                    packages,
                )
            }
            Self::UpdatePath {
                replacement,
                parent,
                replacements,
            } => {
                // comment added to force more readable formatting
                update_path(replacement, parent, replacements, change_set)
            }
            action @ Self::Finish { .. } => action,
        }
    }
}

fn start(
    to_replace: NodePath,
    replacement_token: String,
    children: Children,
) -> ReplaceAction {
    ReplaceAction::CompileToken {
        path: to_replace,
        token: replacement_token,
        children,
        replacements: Vec::new(),
    }
}

fn compile_token(
    path: NodePath,
    token: String,
    children: Children,
    mut replacements: Vec<Replacement>,
    change_set: &mut NewChangeSet,
    errors: &mut Errors,
    packages: &Packages,
) -> ReplaceAction {
    let token = Token {
        text: &token,
        parent: path.parent(),
        sibling_index: path.sibling_index(),
        children,
    };
    let added = token.compile(change_set, errors, packages);

    let replaced = *path.hash();
    let maybe_parent = path.parent().cloned();

    let replacement = Replacement {
        replaced: path,
        replacement: added,
    };

    if let Some(parent) = maybe_parent {
        let parent_node = change_set.nodes().get(parent.hash());

        let mut next_children = parent_node.to_children();
        next_children.replace(&replaced, [added]);

        replacements.push(replacement);

        ReplaceAction::CompileToken {
            path: parent,
            token: parent_node.to_token(packages),
            children: next_children,
            replacements,
        }
    } else {
        ReplaceAction::UpdatePath {
            replacement,
            parent: None,
            replacements,
        }
    }
}

fn update_path(
    replacement: Replacement,
    parent: Option<NodePath>,
    mut replacements: Vec<Replacement>,
    change_set: &mut NewChangeSet,
) -> ReplaceAction {
    let path = NodePath::new(
        replacement.replacement,
        parent,
        replacement.replaced.sibling_index(),
        change_set.nodes(),
    );

    change_set.replace(&replacement.replaced, &path);

    if let Some(replacement) = replacements.pop() {
        ReplaceAction::UpdatePath {
            replacement,
            parent: Some(path),
            replacements,
        }
    } else {
        ReplaceAction::Finish { path }
    }
}

#[derive(Clone)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::Codebase, compiler::replace::ReplaceAction, packages::Packages,
    };

    #[test]
    fn replace_root_node() {
        // Replacing the root node should lead to the root node being
        // recompiled, and the path being updated.

        let packages = Packages::new();
        let mut codebase = Codebase::new();

        let root = codebase.root();
        let mut action = ReplaceAction::Start {
            to_replace: root.path,
            replacement_token: "root".to_string(),
            children: root.node.to_children(),
        };

        codebase.make_change_with_errors(|change_set, errors| {
            action = action.perform(change_set, errors, &packages);
            let ReplaceAction::CompileToken { token, .. } = &action else {
                panic!("Expected recompilation of root node.");
            };
            assert_eq!(token, "root");

            action = action.perform(change_set, errors, &packages);
            let ReplaceAction::UpdatePath { parent, .. } = &action else {
                panic!("Expected path update of root node.")
            };
            assert_eq!(parent, &None);
        });
    }
}
