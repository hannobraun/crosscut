use std::collections::BTreeMap;

use super::FragmentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Replacements {
    pub inner: BTreeMap<FragmentId, FragmentId>,
}

impl Replacements {
    pub fn latest_version_of(&self, id: &FragmentId) -> FragmentId {
        let mut id = id;

        while let Some(replacement) = self.inner.get(id) {
            id = replacement;
        }

        *id
    }
}
