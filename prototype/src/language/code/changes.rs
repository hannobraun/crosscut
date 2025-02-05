use std::collections::{BTreeMap, BTreeSet};

use super::NodePath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Changes {
    change_set: ChangeSet,
}

impl Changes {
    pub fn new() -> Self {
        Self {
            change_set: ChangeSet {
                changes_by_old_version: BTreeMap::new(),
            },
        }
    }

    pub fn latest_version_of(&self, path: NodePath) -> NodePath {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) =
            self.change_set.changes_by_old_version.get(&latest_known)
        {
            already_seen.insert(latest_known);

            if already_seen.contains(later) {
                panic!(
                    "Detected endless loop while searching for latest version."
                );
            } else {
                latest_known = *later;
            }
        }

        latest_known
    }

    pub fn add(&mut self, old: NodePath, new: NodePath) {
        self.change_set.add(old, new);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    changes_by_old_version: BTreeMap<NodePath, NodePath>,
}

impl ChangeSet {
    pub fn add(&mut self, old: NodePath, new: NodePath) {
        self.changes_by_old_version.insert(old, new);
    }
}
