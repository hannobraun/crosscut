use std::collections::{BTreeMap, BTreeSet};

use super::FragmentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Replacements {
    inner: BTreeMap<FragmentId, FragmentId>,
}

impl Replacements {
    pub fn insert_original_and_replacement(
        &mut self,
        original: FragmentId,
        replacement: FragmentId,
    ) {
        self.inner.insert(original, replacement);
    }

    pub fn latest_version_of(&self, id: &FragmentId) -> FragmentId {
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
