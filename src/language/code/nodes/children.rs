use std::{slice, vec};

use super::{NodeHash, NodePath};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Children {
    children: Vec<NodeHash>,
}

impl Children {
    pub fn new(children: impl IntoIterator<Item = NodeHash>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn contains(&self, child: &NodeHash) -> bool {
        self.children.contains(child)
    }

    pub fn has_none(&self) -> bool {
        self.children.is_empty()
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn has_one(&self) -> Option<&NodeHash> {
        if self.children.len() == 1 {
            self.children.first()
        } else {
            None
        }
    }

    pub fn has_multiple(&self) -> Option<impl Iterator<Item = &NodeHash>> {
        if self.children.len() > 1 {
            Some(self.children.iter())
        } else {
            None
        }
    }

    pub fn add(&mut self, to_add: impl IntoIterator<Item = NodeHash>) {
        self.children.extend(to_add);
    }

    pub fn replace(
        &mut self,
        to_replace: &NodeHash,
        replacements: impl IntoIterator<Item = NodeHash>,
    ) {
        let Some(index) = self
            .children
            .iter()
            .enumerate()
            .find_map(|(i, child)| (child == to_replace).then_some(i))
        else {
            panic!("Trying to replace child that is not present.");
        };

        self.children.remove(index);

        let mut index = index;
        for replacement in replacements {
            self.children.insert(index, replacement);
            index += 1;
        }
    }

    pub fn iter(&self) -> slice::Iter<NodeHash> {
        self.children.iter()
    }

    pub fn to_paths(&self) -> impl Iterator<Item = NodePath> {
        self.children.iter().copied().map(|hash| NodePath { hash })
    }
}

impl<const N: usize> From<[NodeHash; N]> for Children {
    fn from(children: [NodeHash; N]) -> Self {
        Self {
            children: children.into(),
        }
    }
}

impl IntoIterator for Children {
    type Item = NodeHash;
    type IntoIter = vec::IntoIter<NodeHash>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
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
    use crate::language::code::{Node, NodeHash, NodeKind};

    use super::Children;

    #[test]
    fn has_one_should_indicate_whether_there_is_one_child() {
        let [a, b, ..] = test_nodes();

        assert!(Children::new([]).has_one().is_none());
        assert!(Children::new([a]).has_one().is_some());
        assert!(Children::new([a, b]).has_one().is_none());
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
            NodeHash::new(&Node::new(
                NodeKind::Error {
                    node: node.to_string(),
                },
                [],
            ))
        })
    }
}
