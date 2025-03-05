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
            removed: BTreeSet::new(),
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
            let Ok(latest) = change_set.latest_version_of(latest_known) else {
                panic!("Detected circular update path in change set.");
            };

            latest_known = latest;
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

    pub fn change_set(&self) -> &ChangeSet {
        self.change_set
    }

    pub fn add(&mut self, to_add: Node) -> NodeHash {
        self.nodes.insert(to_add)
    }

    pub fn remove(&mut self, to_remove: NodePath) {
        self.change_set.removed.insert(to_remove);
    }

    pub fn replace(
        &mut self,
        to_replace: NodePath,
        replacement: Node,
    ) -> NodePath {
        let replacement = NodePath {
            hash: self.nodes.insert(replacement),
            // Once `NodePath` gets more fields, we can just copy those from
            // `to_replace`.
        };

        self.change_set
            .replacements_by_replaced
            .insert(to_replace, replacement);

        replacement
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    removed: BTreeSet<NodePath>,
    replacements_by_replaced: BTreeMap<NodePath, NodePath>,
}

impl ChangeSet {
    pub fn was_removed(&self, removed: &NodePath) -> bool {
        self.removed.contains(removed)
    }

    pub fn was_replaced(&self, replaced: &NodePath) -> Option<&NodePath> {
        self.replacements_by_replaced.get(replaced)
    }

    fn latest_version_of(
        &self,
        path: NodePath,
    ) -> Result<NodePath, CircularDependency> {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) = self.replacements_by_replaced.get(&latest_known)
        {
            already_seen.insert(latest_known);

            if already_seen.contains(later) {
                return Err(CircularDependency);
            } else {
                latest_known = *later;
            }
        }

        Ok(latest_known)
    }
}

struct CircularDependency;

#[cfg(test)]
mod tests {
    use crate::language::code::{Node, NodeKind, NodePath, Nodes};

    use super::Changes;

    #[test]
    fn circular_changes_should_work_correctly() {
        let mut changes = Changes::new();
        let mut nodes = Nodes::new();

        let [node_a, node_b] = ["a", "b"].map(|name| {
            Node::new(
                NodeKind::Error {
                    node: String::from(name),
                },
                None,
            )
        });
        let path_a = {
            let hash = nodes.insert(node_a.clone());
            NodePath { hash }
        };

        let path_b = changes.new_change_set(&mut nodes).replace(path_a, node_b);
        let path_a = changes.new_change_set(&mut nodes).replace(path_b, node_a);

        assert_eq!(changes.latest_version_of(path_a), path_a);
        assert_eq!(changes.latest_version_of(path_b), path_a);
    }
}
