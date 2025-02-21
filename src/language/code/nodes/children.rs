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
        let mut replacements = replacements.into_iter();

        assert!(
            self.children.len() <= 1,
            "Nodes with multiple children are not fully supported yet.",
        );
        assert_eq!(
            self.children.first(),
            Some(to_replace),
            "Trying to replace child that is not present.",
        );

        self.children.clear();
        self.children.extend(replacements.next());

        assert!(
            replacements.next().is_none(),
            "Replacing a child with multiple other children is not supported \
            yet.",
        );
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
