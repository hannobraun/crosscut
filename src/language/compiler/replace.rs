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
        replacements: Vec<Replacement>,
        parent: Option<NodePath>,
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

                replacements.push(Replacement {
                    replaced: path,
                    replacement: added,
                });

                if let Some(parent) = maybe_parent {
                    let parent_node = change_set.nodes().get(parent.hash());

                    let mut next_children = parent_node.to_children();
                    next_children.replace(&replaced, [added]);

                    Self::CompileToken {
                        path: parent,
                        token: parent_node.to_token(packages),
                        children: next_children,
                        replacements,
                    }
                } else {
                    Self::UpdatePath {
                        replacements,
                        parent: None,
                    }
                }
            }
            Self::UpdatePath {
                mut replacements,
                mut parent,
            } => {
                if let Some(replacement) = replacements.pop() {
                    let path = NodePath::new(
                        replacement.replacement,
                        parent.clone(),
                        replacement.replaced.sibling_index(),
                        change_set.nodes(),
                    );

                    change_set.replace(&replacement.replaced, &path);

                    parent = Some(path.clone());

                    Self::UpdatePath {
                        replacements,
                        parent,
                    }
                } else {
                    let Some(path) = parent.clone() else {
                        unreachable!(
                            "There is always at least one replacement, so we \
                            _must_ have set the `parent` at least once in the \
                            code above."
                        );
                    };

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
