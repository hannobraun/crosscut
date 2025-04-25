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
    let replacements = Vec::new();
    let mut next_action = compile_token(
        to_replace,
        replacement_token,
        children,
        replacements,
        change_set,
        errors,
        packages,
    );

    loop {
        next_action = next_action.perform(change_set, errors, packages);

        if let ReplaceAction::Finish { path } = next_action {
            break path;
        }
    }
}

#[derive(Debug)]
enum ReplaceAction {
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
        next_children.replace(&replacement.replaced, replacement.replacement);

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

#[derive(Clone, Debug)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash<Expression>,
}
