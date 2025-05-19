use std::vec;

use super::NodeHash;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct ChildrenOwned {
    pub inner: Vec<NodeHash>,
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
