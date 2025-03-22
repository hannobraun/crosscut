use super::NodePath;

pub struct SyntaxTree {}

impl SyntaxTree {
    pub fn from_root(_: NodePath) -> Self {
        Self {}
    }

    pub fn find_parent_of(self, child: &NodePath) -> Option<NodePath> {
        child.parent().cloned()
    }
}
