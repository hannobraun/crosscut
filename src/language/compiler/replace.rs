use crate::language::{
    code::{Children, Errors, Expression, NewChangeSet, NodeHash, NodePath},
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
    let replacement = token.compile(change_set, errors, packages);

    let replacement = Replacement {
        replaced: path,
        replacement,
    };

    if let Some(parent) = replacement.replaced.parent().cloned() {
        let parent_node = change_set.nodes().get(parent.hash());

        let mut next_children = parent_node.to_children();
        next_children
            .replace(replacement.replaced.hash(), [replacement.replacement]);

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
    replacement: NodeHash<Expression>,
}

#[cfg(test)]
mod tests {
    use crate::language::{
        code::{Codebase, LocatedNode, NodePath},
        compiler::replace::{ReplaceAction, Replacement},
        packages::Packages,
    };

    #[test]
    fn replace_root() {
        // Replacing the root node should lead to the root node being
        // recompiled, and the path being updated.

        let packages = Packages::default();
        let mut codebase = Codebase::new();

        let mut action = ReplaceAction::start(codebase.root(), "root");

        codebase.make_change_with_errors(|change_set, errors| {
            action = action.perform(change_set, errors, &packages);
            assert_eq!(action.expect_compile_token_and_extract_token(), "root");

            action = action.perform(change_set, errors, &packages);
            assert_eq!(
                action
                    .expect_update_path_and_extract_replaced()
                    .distance_from_root(),
                0,
            );

            action
                .perform(change_set, errors, &packages)
                .expect_finish();
        });
    }

    trait ReplaceActionExt {
        fn start(located_node: LocatedNode, replacement_token: &str) -> Self;
        fn expect_compile_token_and_extract_token(&self) -> &str;
        fn expect_update_path_and_extract_replaced(&self) -> &NodePath;
        fn expect_finish(&self);
    }

    impl ReplaceActionExt for ReplaceAction {
        fn start(located_node: LocatedNode, replacement_token: &str) -> Self {
            Self::Start {
                to_replace: located_node.path,
                replacement_token: replacement_token.to_string(),
                children: located_node.node.to_children(),
            }
        }

        #[track_caller]
        fn expect_compile_token_and_extract_token(&self) -> &str {
            let ReplaceAction::CompileToken { token, .. } = self else {
                panic!("Expected `CompileToken`.");
            };

            token
        }

        #[track_caller]
        fn expect_update_path_and_extract_replaced(&self) -> &NodePath {
            let ReplaceAction::UpdatePath {
                replacement: Replacement { replaced, .. },
                ..
            } = self
            else {
                panic!("Expected `UpdatePath`.")
            };

            replaced
        }

        #[track_caller]
        fn expect_finish(&self) {
            let ReplaceAction::Finish { .. } = self else {
                panic!("Expected `Finish`.");
            };
        }
    }
}
