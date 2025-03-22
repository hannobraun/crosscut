use super::NodePath;

pub struct SyntaxTree {}

impl SyntaxTree {
    pub fn find_parent_of(child: &NodePath) -> Option<NodePath> {
        child.parent().cloned()
    }
}
