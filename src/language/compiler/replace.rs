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
            } => Self::CompileToken {
                path: to_replace,
                token: replacement_token,
                children,
                replacements: Vec::new(),
            },
            Self::CompileToken {
                path,
                token,
                children,
                mut replacements,
            } => {
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

                    Self::CompileToken {
                        path: parent,
                        token: parent_node.to_token(packages),
                        children: next_children,
                        replacements,
                    }
                } else {
                    Self::UpdatePath {
                        replacement,
                        parent: None,
                        replacements,
                    }
                }
            }
            Self::UpdatePath {
                replacement,
                parent,
                mut replacements,
            } => {
                let path = NodePath::new(
                    replacement.replacement,
                    parent,
                    replacement.replaced.sibling_index(),
                    change_set.nodes(),
                );

                change_set.replace(&replacement.replaced, &path);

                if let Some(replacement) = replacements.pop() {
                    Self::UpdatePath {
                        replacement,
                        parent: Some(path),
                        replacements,
                    }
                } else {
                    Self::Finish { path }
                }
            }
            action @ Self::Finish { .. } => action,
        }
    }
}

#[derive(Clone)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}
