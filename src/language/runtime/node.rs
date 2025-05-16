use crate::language::code::{Codebase, NodePath};

use super::Value;

#[derive(Clone, Debug)]
pub enum RuntimeNode {
    Generic {
        path: NodePath,
        children_to_evaluate: Vec<NodePath>,
        evaluated_children: Vec<Value>,
    },
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

        Self::Generic {
            path,
            children_to_evaluate,
            evaluated_children,
        }
    }

    pub fn child_was_evaluated(&mut self, output: Value) {
        let Self::Generic {
            evaluated_children, ..
        } = self;
        evaluated_children.push(output);
    }
}
