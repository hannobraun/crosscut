use std::vec;

use super::{Expression, NodeHash, NodePath, RawHash, SiblingIndex};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct Children {
    pub inner: Vec<NodeHash<Expression>>,
}

impl Children {
    pub fn new(
        children: impl IntoIterator<Item = NodeHash<Expression>>,
    ) -> Self {
        let inner = children.into_iter().collect();
        Self { inner }
    }

    pub fn contains_at(
        &self,
        child: &RawHash,
        sibling_index: &SiblingIndex,
    ) -> bool {
        self.inner
            .iter()
            .enumerate()
            .any(|(index, c)| c.raw() == child && index == sibling_index.index)
    }

    pub fn next_index(&self) -> SiblingIndex {
        SiblingIndex {
            index: self.inner.len(),
        }
    }

    pub fn add(&mut self, to_add: NodeHash<Expression>) -> SiblingIndex {
        let index = self.next_index();
        self.inner.push(to_add);
        index
    }

    pub fn replace(
        &mut self,
        to_replace: &NodePath<Expression>,
        replacement: NodeHash<Expression>,
    ) {
        let Some(child) = self
            .inner
            .get_mut(to_replace.sibling_index().unwrap().index)
        else {
            panic!(
                "Trying to replace a child at an index that is not present."
            );
        };

        assert_eq!(
            child,
            to_replace.hash(),
            "Trying to replace a child that is not present."
        );

        *child = replacement;
    }
}

impl<const N: usize> From<[NodeHash<Expression>; N]> for Children {
    fn from(children: [NodeHash<Expression>; N]) -> Self {
        Self {
            inner: children.into(),
        }
    }
}

impl IntoIterator for Children {
    type Item = NodeHash<Expression>;
    type IntoIter = vec::IntoIter<NodeHash<Expression>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
