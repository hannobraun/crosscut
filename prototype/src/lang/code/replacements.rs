use std::collections::BTreeMap;

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
        let mut current_id = id;

        while let Some(replacement) = self.inner.get(current_id) {
            current_id = replacement;
        }

        *current_id
    }
}
