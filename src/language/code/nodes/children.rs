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

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn contains(&self, child: &NodeHash) -> bool {
        self.children.contains(child)
    }

    /// # Access the single child of this node
    ///
    /// Returns `None`, if the node has zero or more than one children.
    pub fn has_one(&self) -> Option<&NodeHash> {
        assert!(
            self.children.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );

        self.children.first()
    }

    pub fn add(&mut self, to_add: NodeHash) {
        assert!(
            self.children.is_empty(),
            "Syntax nodes with multiple children are not fully supported yet.",
        );

        self.children.push(to_add);
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

    pub fn to_paths(&self) -> impl Iterator<Item = NodePath> {
        self.children.iter().copied().map(|hash| NodePath { hash })
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
        self.children.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::language::code::{Node, NodeHash, NodeKind};

    use super::Children;

    #[test]
    fn replace_should_insert_replacements_at_location_of_replaced() {
        let [a, b, c, d, e] = ["a", "b", "c", "d", "e"].map(|node| {
            NodeHash::new(&Node::new(
                NodeKind::Error {
                    node: node.to_string(),
                },
                [],
            ))
        });

        let mut children = Children::new([a, b, c]);
        children.replace(&b, [d, e]);

        assert_eq!(children, Children::new([a, d, e, c]));
    }
}
