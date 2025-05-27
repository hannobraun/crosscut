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
    offset: ChildIndex,
}

impl<T: Form> TypedChildren<T> {
    pub fn new(
        hashes: T::Form<Vec<NodeHash>>,
        offset: impl Into<ChildIndex>,
    ) -> Self {
        let offset = offset.into();
        Self { hashes, offset }
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
            .map(|(index, hash)| {
                let index = self.offset.index + index;
                TypedChild::new(hash, index)
            })
    }

    pub fn to_paths(
        &self,
        parent: &NodePath,
        nodes: &Nodes,
    ) -> impl DoubleEndedIterator<Item = NodePath> + ExactSizeIterator {
        self.iter()
            .map(|child| child.into_path(parent.clone(), nodes))
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
