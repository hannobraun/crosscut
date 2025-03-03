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
            changes_by_old_version: BTreeMap::new(),
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
                    .changes_by_old_version
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
    pub nodes: &'r mut Nodes,
    pub change_set: &'r mut ChangeSet,
}
impl NewChangeSet<'_> {
    pub fn insert(&mut self, node: Node) -> NodeHash {
        self.nodes.insert(node)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    changes_by_old_version: BTreeMap<NodePath, NodePath>,
}

impl ChangeSet {
    pub fn add(&mut self, old: NodePath, new: NodePath) -> &mut Self {
        self.changes_by_old_version.insert(old, new);
        self
    }

    fn latest_version_of(&self, path: NodePath) -> NodePath {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) = self.changes_by_old_version.get(&latest_known) {
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

        let [a, b] = ["a", "b"].map(|node| {
            let node = Node::new(
                NodeKind::Error {
                    node: String::from(node),
                },
                None,
            );
            let hash = nodes.insert(node);
            NodePath { hash }
        });

        changes.new_change_set(&mut nodes).change_set.add(a, b);
        changes.new_change_set(&mut nodes).change_set.add(b, a);

        assert_eq!(changes.latest_version_of(a), a);
        assert_eq!(changes.latest_version_of(b), a);
    }
}
