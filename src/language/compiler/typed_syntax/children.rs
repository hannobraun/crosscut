use crate::{
    language::code::{ChildIndex, NodeHash},
    util::form::{Form, RefMut},
};

pub struct Children<T: Form> {
    hashes: T::Form<Vec<NodeHash>>,
}

impl<T: Form> Children<T> {
    pub fn new(hashes: T::Form<Vec<NodeHash>>, _: usize) -> Self {
        Self { hashes }
    }
}

impl Children<RefMut<'_>> {
    pub fn add(&mut self, child: NodeHash) -> ChildIndex {
        let index = ChildIndex {
            index: self.hashes.len(),
        };
        self.hashes.push(child);
        index
    }
}
