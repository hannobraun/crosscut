use crate::{
    language::code::{ChildIndex, NodeHash, NodePath, Nodes},
    util::form::{Form, Ref, RefMut},
};

pub struct TypedChild {
    hash: NodeHash,
    index: ChildIndex,
}

impl TypedChild {
    pub fn new(hash: NodeHash, index: impl Into<ChildIndex>) -> Self {
        let index = index.into();
        Self { hash, index }
    }

    pub fn into_path(self, parent: NodePath, nodes: &Nodes) -> NodePath {
        NodePath::new(self.hash, Some((parent, self.index)), nodes)
    }
}

pub struct TypedChildren<T: Form> {
    hashes: T::Form<Vec<NodeHash>>,
}

impl<T: Form> TypedChildren<T> {
    pub fn new(hashes: T::Form<Vec<NodeHash>>) -> Self {
        Self { hashes }
    }
}

impl TypedChildren<Ref<'_>> {
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = TypedChild> + ExactSizeIterator {
        self.hashes
            .iter()
            .copied()
            .enumerate()
            .map(|(index, hash)| TypedChild::new(hash, index))
    }
}

impl TypedChildren<RefMut<'_>> {
    pub fn add(&mut self, child: NodeHash) -> ChildIndex {
        let index = ChildIndex {
            index: self.hashes.len(),
        };
        self.hashes.push(child);
        index
    }
}
