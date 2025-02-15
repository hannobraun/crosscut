use crate::language::code::NodePath;

use super::ValueWithSource;

#[derive(Clone, Debug)]
pub struct Context {
    /// # The nodes to be evaluated, sorted from root to leaf
    ///
    /// This is a subset of the full syntax tree. But it is not a tree itself,
    /// just a sequence of syntax node. If any of the nodes had multiple
    /// children (which would turn the sequence into a sub-tree), this would
    /// have caused a separate context to be created.
    pub nodes_from_root: Vec<NodePath>,

    pub active_value: ValueWithSource,
}

impl Context {
    pub fn advance(&mut self) {
        self.nodes_from_root.pop();
    }
}
