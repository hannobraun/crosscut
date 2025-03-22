use super::NodePath;

pub fn find_parent_of(child: &NodePath) -> Option<NodePath> {
    child.parent().cloned()
}
