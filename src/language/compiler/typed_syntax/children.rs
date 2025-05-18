use crate::language::code::{NodeHash, SiblingIndex};

use super::{Form, Ref, RefMut};

pub struct Child<T: Form> {
    hash: T::Form<NodeHash>,
    index: SiblingIndex,
}

impl<T: Form> Child<T> {
    pub fn new(hash: T::Form<NodeHash>, index: usize) -> Self {
        let index = SiblingIndex { index };
        Self { hash, index }
    }
}

impl Child<Ref<'_>> {
    pub fn is(&self, hash: &NodeHash, index: &SiblingIndex) -> bool {
        self.hash == hash && &self.index == index
    }
}

impl Child<RefMut<'_>> {
    pub fn as_ref(&self) -> Child<Ref> {
        Child {
            hash: self.hash,
            index: self.index,
        }
    }

    pub fn replace(
        &mut self,
        replace_hash: &NodeHash,
        replace_index: &SiblingIndex,
        replacement: NodeHash,
    ) -> bool {
        if self.as_ref().is(replace_hash, replace_index) {
            *self.hash = replacement;
            true
        } else {
            false
        }
    }
}

pub struct Children<T: Form> {
    hashes: Vec<T::Form<NodeHash>>,
    offset: SiblingIndex,
}

impl<'r> Children<Ref<'r>> {
    pub fn new(
        hashes: impl IntoIterator<Item = &'r NodeHash>,
        offset: usize,
    ) -> Self {
        let hashes = hashes.into_iter().collect();
        let offset = SiblingIndex { index: offset };

        Self { hashes, offset }
    }

    pub fn contains(&self, hash: &NodeHash, index: &SiblingIndex) -> bool {
        self.hashes
            .iter()
            .copied()
            .enumerate()
            .any(|(i, c)| c == hash && i + self.offset.index == index.index)
    }
}
