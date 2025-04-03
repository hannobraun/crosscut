use crate::language::{
    code::{Children, CodeError, Errors, NewChangeSet, NodeHash, NodePath},
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
    let mut strategy = ReplacementStrategy {
        next_to_replace: to_replace.clone(),
        next_token: replacement_token.to_string(),
        next_children: children,
        added_nodes: Vec::new(),
    };

    loop {
        let token = strategy.next_action();
        let (node, maybe_error) = token.compile(change_set.nodes(), packages);

        let added = change_set.add(node);

        strategy.added_nodes.push(NodeAddedDuringReplacement {
            replaced: strategy.next_to_replace.clone(),
            added,
            maybe_error,
        });

        if let Some(parent_path) = strategy.next_to_replace.parent().cloned() {
            let parent_node = change_set.nodes().get(parent_path.hash());

            strategy.next_token = parent_node.to_token(packages);
            strategy.next_children = parent_node.to_children();

            strategy
                .next_children
                .replace(strategy.next_to_replace.hash(), [added]);

            strategy.next_to_replace = parent_path;

            continue;
        } else {
            break;
        };
    }

    let mut initial_replacement = None;
    let mut parent = None;

    while let Some(NodeAddedDuringReplacement {
        replaced,
        added,
        maybe_error,
    }) = strategy.added_nodes.pop()
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

pub struct ReplacementStrategy {
    pub next_to_replace: NodePath,
    pub next_token: String,
    pub next_children: Children,
    pub added_nodes: Vec<NodeAddedDuringReplacement>,
}

impl ReplacementStrategy {
    pub fn next_action(&self) -> Token {
        Token {
            text: &self.next_token,
            parent: self.next_to_replace.parent(),
            sibling_index: self.next_to_replace.sibling_index(),
            children: self.next_children.clone(),
        }
    }
}

pub struct NodeAddedDuringReplacement {
    pub replaced: NodePath,
    pub added: NodeHash,
    pub maybe_error: Option<CodeError>,
}
