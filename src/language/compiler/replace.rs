use std::mem;

use crate::language::{
    code::{Children, Errors, NewChangeSet, NodeHash, NodePath, Nodes},
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

    while let Some(action) = strategy.next_action(change_set.nodes()) {
        match action {
            ReplacementAction::CompileToken { action } => {
                let (node, maybe_error) =
                    action.token().compile(change_set.nodes(), packages);

                let added = change_set.add(node);
                if let Some(error) = maybe_error {
                    errors.insert(added, error);
                }

                action.provide_replacement(added, change_set.nodes(), packages);
            }
            ReplacementAction::UpdatePath {
                replaced,
                replacement,
            } => {
                change_set.replace(&replaced, &replacement);
            }
        }
    }

    let ReplacementStrategy::UpdatingPathsAfterReplacement {
        parent: initial_replacement,
        ..
    } = strategy
    else {
        unreachable!(
            "Strategy is put into this state after replacement has propagated \
            to root."
        );
    };

    let Some(path) = initial_replacement else {
        unreachable!(
            "The loop above is executed at least once. The variable must have \
            been set."
        );
    };

    path
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

impl ReplacementStrategy {
    fn next_action(&mut self, nodes: &Nodes) -> Option<ReplacementAction> {
        match self {
            strategy @ Self::PropagatingReplacementToRoot { .. } => {
                Some(ReplacementAction::CompileToken {
                    action: CompileToken { strategy },
                })
            }
            Self::UpdatingPathsAfterReplacement {
                replacements: added_nodes,
                parent,
            } => added_nodes.pop().map(|node| {
                let replacement = NodePath::new(
                    node.replacement,
                    parent.clone(),
                    node.replaced.sibling_index(),
                    nodes,
                );

                *parent = Some(replacement.clone());

                ReplacementAction::UpdatePath {
                    replaced: node.replaced,
                    replacement,
                }
            }),
            Self::PlaceholderState => {
                unreachable!("Strategy is never left in placeholder state.");
            }
        }
    }
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

    fn provide_replacement(
        self,
        replacement: NodeHash,
        nodes: &Nodes,
        packages: &Packages,
    ) {
        let strategy =
            mem::replace(self.strategy, ReplacementStrategy::PlaceholderState);

        let ReplacementStrategy::PropagatingReplacementToRoot {
            next_to_replace,
            mut replacements,
            ..
        } = strategy
        else {
            unreachable!(
                "This action only exists while replacement strategy is in this \
                state."
            );
        };

        let replaced = *next_to_replace.hash();
        let maybe_parent = next_to_replace.parent().cloned();

        replacements.push(Replacement {
            replaced: next_to_replace,
            replacement,
        });

        if let Some(parent) = maybe_parent {
            let parent_node = nodes.get(parent.hash());

            let mut next_children = parent_node.to_children();
            next_children.replace(&replaced, [replacement]);

            *self.strategy =
                ReplacementStrategy::PropagatingReplacementToRoot {
                    next_to_replace: parent,
                    next_token: parent_node.to_token(packages),
                    next_children,
                    replacements,
                };
        } else {
            *self.strategy =
                ReplacementStrategy::UpdatingPathsAfterReplacement {
                    replacements,
                    parent: None,
                };
        }
    }
}
