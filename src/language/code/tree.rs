use super::{NodePath, Nodes};

pub struct SyntaxTree {}

impl SyntaxTree {
    pub fn from_root(_: NodePath) -> Self {
        Self {}
    }

    pub fn find_parent_of(
        self,
        child: &NodePath,
        _: &Nodes,
    ) -> Option<NodePath> {
        child.parent().cloned()
    }
}
