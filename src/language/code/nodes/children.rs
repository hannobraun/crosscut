use std::{slice, vec};

use super::{Expression, NodeHash, NodePath, SiblingIndex};

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

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn contains_at(
        &self,
        child: &NodeHash<Expression>,
        sibling_index: &SiblingIndex,
    ) -> bool {
        self.inner
            .iter()
            .enumerate()
            .any(|(index, c)| (c == child && index == sibling_index.index))
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn is_single_child(&self) -> Option<&NodeHash<Expression>> {
        if self.inner.len() == 1 {
            self.inner.first()
        } else {
            None
        }
    }

    pub fn is_multiple_children(
        &self,
    ) -> Option<impl Iterator<Item = &NodeHash<Expression>>> {
        if self.inner.len() > 1 {
            Some(self.inner.iter())
        } else {
            None
        }
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
        to_replace: &NodePath,
        replacement: NodeHash<Expression>,
    ) {
        let Some(child) = self.inner.get(to_replace.sibling_index().index)
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

        let Some(index) =
            self.inner.iter().enumerate().find_map(|(i, child)| {
                (child == to_replace.hash()).then_some(i)
            })
        else {
            panic!("Trying to replace child that is not present.");
        };

        self.inner.remove(index);
        self.inner.insert(index, replacement);
    }

    pub fn iter(&self) -> slice::Iter<NodeHash<Expression>> {
        self.inner.iter()
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

impl<'r> IntoIterator for &'r Children {
    type Item = &'r NodeHash<Expression>;
    type IntoIter = slice::Iter<'r, NodeHash<Expression>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::{Expression, NodeHash};

    use super::Children;

    #[test]
    fn has_one_should_indicate_whether_there_is_one_child() {
        let [a, b, ..] = test_nodes();

        assert!(Children::new([]).is_single_child().is_none());
        assert!(Children::new([a]).is_single_child().is_some());
        assert!(Children::new([a, b]).is_single_child().is_none());
    }

    fn test_nodes() -> [NodeHash<Expression>; 5] {
        ["a", "b", "c", "d", "e"].map(|node| {
            NodeHash::new(&Expression::Error {
                node: node.to_string(),
                children: Children::new([]),
            })
        })
    }
}
