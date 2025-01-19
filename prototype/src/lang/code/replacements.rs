use std::collections::BTreeMap;

use super::FragmentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Replacements {
    pub inner: BTreeMap<FragmentId, FragmentId>,
}
