use crate::language::code::{NodeHash, NodePath, SiblingIndex};

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
    hashes: T::Form<Vec<NodeHash>>,
    offset: SiblingIndex,
}

impl<T: Form> Children<T> {
    pub fn new(hashes: T::Form<Vec<NodeHash>>, offset: usize) -> Self {
        let offset = SiblingIndex { index: offset };
        Self { hashes, offset }
    }
}

impl Children<Ref<'_>> {
    pub fn contains(&self, hash: &NodeHash, index: &SiblingIndex) -> bool {
        self.hashes
            .iter()
            .enumerate()
            .any(|(i, c)| c == hash && i + self.offset.index == index.index)
    }
}

impl Children<RefMut<'_>> {
    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement: NodeHash,
    ) -> bool {
        if let Some(child) = self.hashes.get_mut(
            to_replace.sibling_index().unwrap().index - self.offset.index,
        ) {
            if child == to_replace.hash() {
                *child = replacement;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
