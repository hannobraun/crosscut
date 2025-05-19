use std::vec;

use super::{NodeHash, NodePath, SiblingIndex};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct ChildrenOwned {
    pub inner: Vec<NodeHash>,
}

impl ChildrenOwned {
    #[cfg(test)]
    pub fn new(children: impl IntoIterator<Item = NodeHash>) -> Self {
        let inner = children.into_iter().collect();
        Self { inner }
    }

    pub fn contains_at(
        &self,
        child: &NodeHash,
        sibling_index: &SiblingIndex,
        offset: usize,
    ) -> bool {
        self.inner.iter().enumerate().any(|(index, c)| {
            c == child && index + offset == sibling_index.index
        })
    }

    pub fn next_index(&self) -> SiblingIndex {
        SiblingIndex {
            index: self.inner.len(),
        }
    }

    pub fn add(&mut self, to_add: NodeHash) -> SiblingIndex {
        let index = self.next_index();
        self.inner.push(to_add);
        index
    }

    #[must_use]
    pub fn replace(
        &mut self,
        to_replace: &NodePath,
        replacement: NodeHash,
        offset: usize,
    ) -> bool {
        if let Some(child) = self
            .inner
            .get_mut(to_replace.sibling_index().unwrap().index - offset)
        {
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

impl<const N: usize> From<[NodeHash; N]> for ChildrenOwned {
    fn from(children: [NodeHash; N]) -> Self {
        Self {
            inner: children.into(),
        }
    }
}

impl IntoIterator for ChildrenOwned {
    type Item = NodeHash;
    type IntoIter = vec::IntoIter<NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
