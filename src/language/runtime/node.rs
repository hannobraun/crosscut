use crate::language::code::{Codebase, NodePath};

use super::Value;

#[derive(Clone, Debug)]
pub struct RuntimeNode {
    pub path: NodePath,
    pub children_to_evaluate: Vec<NodePath>,
    pub evaluated_children: Vec<Value>,
}

impl RuntimeNode {
    pub fn new(path: NodePath, codebase: &Codebase) -> Self {
        let children_to_evaluate = codebase
            .node_at(&path)
            .inputs(codebase.nodes())
            .map(|located_node| located_node.path)
            .rev()
            .collect();
        let evaluated_children = Vec::new();

        Self {
            path,
            children_to_evaluate,
            evaluated_children,
        }
    }
}
