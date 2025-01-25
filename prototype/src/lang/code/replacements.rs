use std::collections::{BTreeMap, BTreeSet};

use super::NodeId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Replacements {
    inner: BTreeMap<NodeId, NodeId>,
}

impl Replacements {
    pub fn insert_original_and_replacement(
        &mut self,
        original: NodeId,
        replacement: NodeId,
    ) {
        if original == replacement {
            // It seems like this check should maybe be an assertion instead,
            // but it's actually quite easy to get into a situation where a
            // token gets compiled unchanged, which will lead to this case. And
            // there might actually not be an easier place to catch than, than
            // here.
            return;
        }

        self.inner.insert(original, replacement);
    }

    pub fn latest_version_of(&self, id: &NodeId) -> NodeId {
        let mut already_seen = BTreeSet::new();
        let mut current_id = id;

        while let Some(replacement) = self.inner.get(current_id) {
            if already_seen.contains(replacement) {
                unreachable!(
                    "Detected endless loop while searching for latest version \
                    of {id:?}. IDs found: {already_seen:?}"
                );
            }

            already_seen.insert(id);
            current_id = replacement;
        }

        *current_id
    }
}
