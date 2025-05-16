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
        let syntax_node = codebase.node_at(&path);

        let children_to_evaluate = syntax_node
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

    pub fn child_was_evaluated(&mut self, value: Value) {
        match self {
            Self::Generic {
                evaluated_children, ..
            } => {
                evaluated_children.push(value);
            }
        }
    }
}
