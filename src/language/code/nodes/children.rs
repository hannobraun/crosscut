use std::{slice, vec};

use super::NodeHash;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Children {
    pub inner: Vec<NodeHash>,
}

impl Children {
    pub fn new(children: impl IntoIterator<Item = NodeHash>) -> Self {
        let inner = children.into_iter().collect();
        Self { inner }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn is_single_child(&self) -> Option<&NodeHash> {
        if self.inner.len() == 1 {
            self.inner.first()
        } else {
            None
        }
    }

    pub fn is_multiple_children(
        &self,
    ) -> Option<impl Iterator<Item = &NodeHash>> {
        if self.inner.len() > 1 {
            Some(self.inner.iter())
        } else {
            None
        }
    }

    pub fn add(&mut self, to_add: NodeHash) -> usize {
        let index = self.inner.len();
        self.inner.push(to_add);
        index
    }

    pub fn replace(
        &mut self,
        to_replace: &NodeHash,
        replacements: impl IntoIterator<Item = NodeHash>,
    ) {
        let Some(index) = self
            .inner
            .iter()
            .enumerate()
            .find_map(|(i, child)| (child == to_replace).then_some(i))
        else {
            panic!("Trying to replace child that is not present.");
        };

        self.inner.remove(index);

        let mut index = index;
        for replacement in replacements {
            self.inner.insert(index, replacement);
            index += 1;
        }
    }

    pub fn iter(&self) -> slice::Iter<NodeHash> {
        self.inner.iter()
    }
}

impl<const N: usize> From<[NodeHash; N]> for Children {
    fn from(children: [NodeHash; N]) -> Self {
        Self {
            inner: children.into(),
        }
    }
}

impl IntoIterator for Children {
    type Item = NodeHash;
    type IntoIter = vec::IntoIter<NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'r> IntoIterator for &'r Children {
    type Item = &'r NodeHash;
    type IntoIter = slice::Iter<'r, NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::{Node, NodeHash};

    use super::Children;

    #[test]
    fn has_one_should_indicate_whether_there_is_one_child() {
        let [a, b, ..] = test_nodes();

        assert!(Children::new([]).is_single_child().is_none());
        assert!(Children::new([a]).is_single_child().is_some());
        assert!(Children::new([a, b]).is_single_child().is_none());
    }

    #[test]
    fn replace_should_insert_replacements_at_location_of_replaced() {
        let [a, b, c, d, e] = test_nodes();

        let mut children = Children::new([a, b, c]);
        children.replace(&b, [d, e]);

        assert_eq!(children, Children::new([a, d, e, c]));
    }

    fn test_nodes() -> [NodeHash; 5] {
        ["a", "b", "c", "d", "e"].map(|node| {
            NodeHash::new(&Node::Error {
                node: node.to_string(),
                children: Children::new([]),
            })
        })
    }
}
