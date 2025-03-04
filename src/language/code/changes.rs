use std::collections::{BTreeMap, BTreeSet};

use super::{Node, NodeHash, NodePath, Nodes};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Changes {
    change_sets: Vec<ChangeSet>,
}

impl Changes {
    pub fn new() -> Self {
        Self {
            change_sets: Vec::new(),
        }
    }

    pub fn new_change_set<'r>(
        &'r mut self,
        nodes: &'r mut Nodes,
    ) -> NewChangeSet<'r> {
        self.change_sets.push(ChangeSet {
            replacements_by_replaced: BTreeMap::new(),
        });

        let Some(change_set) = self.change_sets.last_mut() else {
            unreachable!("Just pushed a change set. One _must_ be available.");
        };

        NewChangeSet { nodes, change_set }
    }

    pub fn latest_version_of(&self, path: NodePath) -> NodePath {
        let Some(i) = self.change_sets.iter().enumerate().rev().find_map(
            |(i, change_set)| {
                change_set
                    .replacements_by_replaced
                    .contains_key(&path)
                    .then_some(i)
            },
        ) else {
            return path;
        };

        let mut latest_known = path;

        for change_set in &self.change_sets[i..] {
            latest_known = change_set.latest_version_of(latest_known);
        }

        latest_known
    }
}

pub struct NewChangeSet<'r> {
    nodes: &'r mut Nodes,
    change_set: &'r mut ChangeSet,
}
impl NewChangeSet<'_> {
    pub fn nodes(&self) -> &Nodes {
        self.nodes
    }

    pub fn replace(
        &mut self,
        to_replace: NodePath,
        replacement: Node,
    ) -> NodeHash {
        let hash = self.nodes.insert(replacement);
        self.change_set.add(to_replace, NodePath { hash });
        hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ChangeSet {
    replacements_by_replaced: BTreeMap<NodePath, NodePath>,
}

impl ChangeSet {
    pub fn add(&mut self, old: NodePath, new: NodePath) -> &mut Self {
        self.replacements_by_replaced.insert(old, new);
        self
    }

    fn latest_version_of(&self, path: NodePath) -> NodePath {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) = self.replacements_by_replaced.get(&latest_known)
        {
            already_seen.insert(latest_known);

            if already_seen.contains(later) {
                unreachable!(
                    "Detected endless loop while searching for latest version
                    of node within change set.\n\
                    \n\
                    This should never happen, unless a caller puts a circular \
                    change graph into a single change set. Since `Codebase` \
                    not expose change sets to its callers, this is a bug \
                    inside of `Codebase`."
                );
            } else {
                latest_known = *later;
            }
        }

        latest_known
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::{Node, NodeKind, NodePath, Nodes};

    use super::Changes;

    #[test]
    fn circular_changes_should_work_correctly() {
        let mut changes = Changes::new();
        let mut nodes = Nodes::new();

        let [node_a, node_b] = ["a", "b"].map(|node| {
            Node::new(
                NodeKind::Error {
                    node: String::from(node),
                },
                None,
            )
        });
        let [path_a, path_b] = [node_a, node_b].map(|node| {
            let hash = nodes.insert(node);
            NodePath { hash }
        });

        changes
            .new_change_set(&mut nodes)
            .change_set
            .add(path_a, path_b);
        changes
            .new_change_set(&mut nodes)
            .change_set
            .add(path_b, path_a);

        assert_eq!(changes.latest_version_of(path_a), path_a);
        assert_eq!(changes.latest_version_of(path_b), path_a);
    }
}
