use std::mem;

use crate::language::{
    code::{Children, Errors, NewChangeSet, NodeHash, NodePath},
    packages::Packages,
};

use super::token::Token;

pub fn replace_node_and_update_parents(
    to_replace: &NodePath,
    replacement_token: &str,
    children: Children,
    packages: &Packages,
    change_set: &mut NewChangeSet,
    errors: &mut Errors,
) -> NodePath {
    let mut strategy = ReplacementState::PropagatingReplacementToRoot {
        next_to_replace: to_replace.clone(),
        next_token: replacement_token.to_string(),
        next_children: children,
        replacements: Vec::new(),
    };

    loop {
        let next_action = match mem::replace(
            &mut strategy,
            ReplacementState::Placeholder,
        ) {
            ReplacementState::PropagatingReplacementToRoot {
                next_to_replace,
                next_token,
                next_children,
                replacements,
            } => ReplaceAction::CompileToken {
                next_to_replace,
                next_token,
                next_children,
                replacements,
            },
            ReplacementState::UpdatingPathsAfterReplacement {
                mut replacements,
                mut parent,
            } => {
                let next_action = if let Some(node) = replacements.pop() {
                    let replacement = NodePath::new(
                        node.replacement,
                        parent.clone(),
                        node.replaced.sibling_index(),
                        change_set.nodes(),
                    );

                    parent = Some(replacement.clone());

                    ReplaceAction::UpdatePath {
                        replacements,
                        parent,
                        replaced: node.replaced,
                        replacement,
                    }
                } else {
                    let Some(path) = parent.clone() else {
                        unreachable!(
                            "There is always at least one replacement, so we \
                            _must_ have set the `parent` at least once in the \
                            code above."
                        );
                    };

                    ReplaceAction::Finish { path }
                };

                next_action
            }
            ReplacementState::Placeholder => {
                unreachable!("Strategy is never left in placeholder state.");
            }
        };

        match next_action {
            ReplaceAction::CompileToken {
                next_to_replace,
                next_token,
                next_children,
                mut replacements,
            } => {
                let token = Token {
                    text: &next_token,
                    parent: next_to_replace.parent(),
                    sibling_index: next_to_replace.sibling_index(),
                    children: next_children.clone(),
                };
                let added = token.compile(change_set, errors, packages);

                let replaced = *next_to_replace.hash();
                let maybe_parent = next_to_replace.parent().cloned();

                replacements.push(Replacement {
                    replaced: next_to_replace,
                    replacement: added,
                });

                if let Some(parent) = maybe_parent {
                    let parent_node = change_set.nodes().get(parent.hash());

                    let mut next_children = parent_node.to_children();
                    next_children.replace(&replaced, [added]);

                    strategy = ReplacementState::PropagatingReplacementToRoot {
                        next_to_replace: parent,
                        next_token: parent_node.to_token(packages),
                        next_children,
                        replacements,
                    };
                } else {
                    strategy =
                        ReplacementState::UpdatingPathsAfterReplacement {
                            replacements,
                            parent: None,
                        };
                };
            }
            ReplaceAction::UpdatePath {
                replacements,
                parent,
                replaced,
                replacement,
            } => {
                change_set.replace(&replaced, &replacement);

                strategy = ReplacementState::UpdatingPathsAfterReplacement {
                    replacements,
                    parent,
                };
            }
            ReplaceAction::Finish { path } => {
                break path;
            }
        }
    }
}

enum ReplacementState {
    PropagatingReplacementToRoot {
        next_to_replace: NodePath,
        next_token: String,
        next_children: Children,
        replacements: Vec<Replacement>,
    },
    UpdatingPathsAfterReplacement {
        replacements: Vec<Replacement>,
        parent: Option<NodePath>,
    },
    Placeholder,
}

#[derive(Clone)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}

enum ReplaceAction {
    CompileToken {
        next_to_replace: NodePath,
        next_token: String,
        next_children: Children,
        replacements: Vec<Replacement>,
    },
    UpdatePath {
        replacements: Vec<Replacement>,
        parent: Option<NodePath>,
        replaced: NodePath,
        replacement: NodePath,
    },
    Finish {
        path: NodePath,
    },
}
