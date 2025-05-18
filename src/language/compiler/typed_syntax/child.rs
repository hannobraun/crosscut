use crate::language::code::{NodeHash, SiblingIndex};

use super::{Form, NodeRef};

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

impl Child<NodeRef<'_>> {
    pub fn is(&self, hash: &NodeHash, index: &SiblingIndex) -> bool {
        self.hash == hash && &self.index == index
    }
}
