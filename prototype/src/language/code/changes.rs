use std::collections::{BTreeMap, BTreeSet};

use super::NodePath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Changes {
    pub inner: BTreeMap<NodePath, NodePath>,
}

impl Changes {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn latest_version_of(&self, path: NodePath) -> NodePath {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) = self.inner.get(&latest_known) {
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
        self.inner.insert(old, new);
    }
}
