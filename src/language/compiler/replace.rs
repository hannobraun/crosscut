use std::mem;

use crate::language::{
    code::{
        Children, CodeError, Errors, NewChangeSet, NodeHash, NodePath, Nodes,
    },
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
        added_nodes: Vec::new(),
    };

    loop {
        let ReplacementAction::CompileToken { action } = strategy.next_action();

        let (node, maybe_error) =
            action.token().compile(change_set.nodes(), packages);
        let added = change_set.add(node);

        if action.provide_added_node(
            added,
            maybe_error,
            change_set.nodes(),
            packages,
        ) {
            continue;
        } else {
            break;
        }
    }

    let ReplacementStrategy::UpdatingPathsAfterReplacement { mut added_nodes } =
        strategy
    else {
        unreachable!(
            "Strategy is put into this state after replacement has propagated \
            to root."
        );
    };

    let mut initial_replacement = None;
    let mut parent = None;

    while let Some(NodeAddedDuringReplacement {
        replaced,
        added,
        maybe_error,
    }) = added_nodes.pop()
    {
        let path = NodePath::new(
            added,
            parent,
            replaced.sibling_index(),
            change_set.nodes(),
        );
        parent = Some(path.clone());

        change_set.replace(&replaced, &path);

        initial_replacement = Some(path.clone());

        if let Some(error) = maybe_error {
            errors.insert(*path.hash(), error);
        }
    }

    let Some(path) = initial_replacement else {
        unreachable!(
            "The loop above is executed at least once. The variable must have \
            been set."
        );
    };

    path
}

#[derive(Default)]
enum ReplacementStrategy {
    PropagatingReplacementToRoot {
        next_to_replace: NodePath,
        next_token: String,
        next_children: Children,
        added_nodes: Vec<NodeAddedDuringReplacement>,
    },
    UpdatingPathsAfterReplacement {
        added_nodes: Vec<NodeAddedDuringReplacement>,
    },

    #[default]
    Placeholder,
}

impl ReplacementStrategy {
    fn next_action(&mut self) -> ReplacementAction {
        ReplacementAction::CompileToken {
            action: CompileToken { strategy: self },
        }
    }
}

#[derive(Clone)]
struct NodeAddedDuringReplacement {
    replaced: NodePath,
    added: NodeHash,
    maybe_error: Option<CodeError>,
}

enum ReplacementAction<'r> {
    CompileToken { action: CompileToken<'r> },
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
            added_nodes: _,
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

    fn provide_added_node(
        self,
        added: NodeHash,
        maybe_error: Option<CodeError>,
        nodes: &Nodes,
        packages: &Packages,
    ) -> bool {
        let ReplacementStrategy::PropagatingReplacementToRoot {
            next_to_replace,
            mut added_nodes,
            ..
        } = mem::take(self.strategy)
        else {
            unreachable!(
                "This action only exists while replacement strategy is in this \
                state."
            );
        };

        let replaced = *next_to_replace.hash();
        let maybe_parent = next_to_replace.parent().cloned();

        added_nodes.push(NodeAddedDuringReplacement {
            replaced: next_to_replace,
            added,
            maybe_error,
        });

        if let Some(parent) = maybe_parent {
            let parent_node = nodes.get(parent.hash());

            let mut next_children = parent_node.to_children();
            next_children.replace(&replaced, [added]);

            *self.strategy =
                ReplacementStrategy::PropagatingReplacementToRoot {
                    next_to_replace: parent,
                    next_token: parent_node.to_token(packages),
                    next_children,
                    added_nodes,
                };
            true
        } else {
            *self.strategy =
                ReplacementStrategy::UpdatingPathsAfterReplacement {
                    added_nodes,
                };
            false
        }
    }
}
