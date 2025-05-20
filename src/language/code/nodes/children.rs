use std::fmt;

use crate::util::form::{Form, Ref};

use super::NodeHash;

pub struct Children<T: Form> {
    pub hashes: Vec<T::Form<NodeHash>>,
}

impl Children<Ref<'_>> {
    pub fn contains(&self, hash: &NodeHash, index: &ChildIndex) -> bool {
        self.hashes
            .iter()
            .copied()
            .enumerate()
            .any(|(i, c)| c == hash && i == index.index)
    }
}

/// # The index of a node among its siblings
///
/// ## Implementation Note
///
/// I'm a bit concerned with the use of `usize` here, as it could lead to
/// problems when serializing `Codebase`. But using something else makes some
/// other code much harder to write. I'd basically have to re-implement
/// `iter::Enumerate`, including its implementation of `DoubleEndedIterator, for
/// `u32` or whatever.
///
/// For now, this works. But it might have to change going forward.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct ChildIndex {
    pub index: usize,
}

impl fmt::Display for ChildIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}
