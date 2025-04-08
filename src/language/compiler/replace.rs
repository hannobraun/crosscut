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
    let mut next_action = ReplaceAction::Start {
        next_to_replace: to_replace.clone(),
        next_token: replacement_token.to_string(),
        next_children: children,
    };

    loop {
        next_action = match next_action {
            ReplaceAction::Start {
                next_to_replace,
                next_token,
                next_children,
            } => ReplaceAction::CompileToken {
                next_to_replace,
                next_token,
                next_children,
                replacements: Vec::new(),
            },
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

                    ReplaceAction::CompileToken {
                        next_to_replace: parent,
                        next_token: parent_node.to_token(packages),
                        next_children,
                        replacements,
                    }
                } else {
                    ReplaceAction::UpdatePath {
                        replacements,
                        parent: None,
                    }
                }
            }
            ReplaceAction::UpdatePath {
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

                    ReplaceAction::UpdatePath {
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

                    ReplaceAction::Finish { path }
                }
            }
            ReplaceAction::Finish { path } => {
                break path;
            }
        };
    }
}

enum ReplaceAction {
    Start {
        next_to_replace: NodePath,
        next_token: String,
        next_children: Children,
    },
    CompileToken {
        next_to_replace: NodePath,
        next_token: String,
        next_children: Children,
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

#[derive(Clone)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}
