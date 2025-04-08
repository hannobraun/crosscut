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
    let mut strategy = ReplacementStrategy::PropagatingReplacementToRoot {
        next_to_replace: to_replace.clone(),
        next_token: replacement_token.to_string(),
        next_children: children,
        replacements: Vec::new(),
    };

    loop {
        let next_action = match &mut strategy {
            strategy @ ReplacementStrategy::PropagatingReplacementToRoot {
                ..
            } => ReplacementAction::CompileToken {
                action: CompileToken { strategy },
            },
            ReplacementStrategy::UpdatingPathsAfterReplacement {
                replacements,
                parent,
            } => {
                if let Some(node) = replacements.pop() {
                    let replacement = NodePath::new(
                        node.replacement,
                        parent.clone(),
                        node.replaced.sibling_index(),
                        change_set.nodes(),
                    );

                    *parent = Some(replacement.clone());

                    ReplacementAction::UpdatePath {
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

                    ReplacementAction::Finish { path }
                }
            }
            ReplacementStrategy::PlaceholderState => {
                unreachable!("Strategy is never left in placeholder state.");
            }
        };

        match next_action {
            ReplacementAction::CompileToken { action } => {
                let added =
                    action.token().compile(change_set, errors, packages);

                let strategy = mem::replace(
                    action.strategy,
                    ReplacementStrategy::PlaceholderState,
                );

                let ReplacementStrategy::PropagatingReplacementToRoot {
                    next_to_replace,
                    mut replacements,
                    ..
                } = strategy
                else {
                    unreachable!(
                        "This action only exists while replacement strategy is \
                        in this state."
                    );
                };

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

                    *action.strategy =
                        ReplacementStrategy::PropagatingReplacementToRoot {
                            next_to_replace: parent,
                            next_token: parent_node.to_token(packages),
                            next_children,
                            replacements,
                        };
                } else {
                    *action.strategy =
                        ReplacementStrategy::UpdatingPathsAfterReplacement {
                            replacements,
                            parent: None,
                        };
                };
            }
            ReplacementAction::UpdatePath {
                replaced,
                replacement,
            } => {
                change_set.replace(&replaced, &replacement);
            }
            ReplacementAction::Finish { path } => {
                break path;
            }
        }
    }
}

enum ReplacementStrategy {
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
    PlaceholderState,
}

#[derive(Clone)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}

enum ReplacementAction<'r> {
    CompileToken {
        action: CompileToken<'r>,
    },
    UpdatePath {
        replaced: NodePath,
        replacement: NodePath,
    },
    Finish {
        path: NodePath,
    },
}

struct CompileToken<'r> {
    strategy: &'r mut ReplacementStrategy,
}

impl CompileToken<'_> {
    fn token(&self) -> Token {
        let ReplacementStrategy::PropagatingReplacementToRoot {
            next_to_replace,
            next_token,
            next_children,
            replacements: _,
        } = &self.strategy
        else {
            unreachable!(
                "This action only exists while replacement strategy is in this \
                state."
            );
        };

        Token {
            text: next_token,
            parent: next_to_replace.parent(),
            sibling_index: next_to_replace.sibling_index(),
            children: next_children.clone(),
        }
    }
}
