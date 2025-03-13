use crate::language::code::NodePath;

use super::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    pub next: Option<ContextNode>,
    pub active_value: Value,
}

impl Context {
    pub fn advance(&mut self) {
        self.next = self
            .next
            .take()
            .and_then(|next| next.parent.map(|child| *child));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextNode {
    pub syntax_node: NodePath,
    pub parent: Option<Box<ContextNode>>,
}
